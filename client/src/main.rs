// Copyright (C) 2018-2019, Cloudflare, Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//
//     * Redistributions in binary form must reproduce the above copyright
//       notice, this list of conditions and the following disclaimer in the
//       documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
// THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
// PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#[macro_use]
extern crate log;

use ring::rand::*;

const MAX_DATAGRAM_SIZE: usize = 1350;

const HTTP_REQ_STREAM_ID: u64 = 4;

fn main() {
    let mut buf = [0; 65535];
    let mut out = [0; MAX_DATAGRAM_SIZE];

    // let mut args = std::env::args();
    //
    // let cmd = &args.next().unwrap();
    //
    // if args.len() != 1 {
    //     println!("Usage: {cmd} URL");
    //     println!("\nSee tools/apps/ for more complete implementations.");
    //     return;
    // }

    let mut url = url::Url::parse("https://127.0.0.1:4433/").unwrap();
    // let url = url::Url::parse(&args.next().unwrap()).unwrap();

    // Setup the event loop.
    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(1024);

    // Resolve server address.
    let peer_addr = url.socket_addrs(|| None).unwrap()[0];
    println!("{}", peer_addr.to_string());

    // Bind to INADDR_ANY or IN6ADDR_ANY depending on the IP family of the
    // server address. This is needed on macOS and BSD variants that don't
    // support binding to IN6ADDR_ANY for both v4 and v6.
    let bind_addr = match peer_addr {
        std::net::SocketAddr::V4(_) => "0.0.0.0:0",
        std::net::SocketAddr::V6(_) => "[::]:0",
    };

    // Create the UDP socket backing the QUIC connection, and register it with
    // the event loop.
    let mut socket =
        mio::net::UdpSocket::bind(bind_addr.parse().unwrap()).unwrap();
    poll.registry()
        .register(&mut socket, mio::Token(0), mio::Interest::READABLE)
        .unwrap();

    // Create the configuration for the QUIC connection.
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();

    // *CAUTION*: this should not be set to `false` in production!!!
    config.verify_peer(false);

    config
        .set_application_protos(&[
            b"hq-interop",
            b"hq-29",
            b"hq-28",
            b"hq-27",
            b"http/0.9",
        ])
        .unwrap();

    config.set_max_idle_timeout(5000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_initial_max_streams_uni(100);
    config.set_disable_active_migration(true);

    // Generate a random source connection ID for the connection.
    let mut scid = [0; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new().fill(&mut scid[..]).unwrap();

    let scid = quiche::ConnectionId::from_ref(&scid);

    // Get local address.
    let local_addr = socket.local_addr().unwrap();

    // Create a QUIC connection and initiate handshake.
    let mut conn =
        quiche::connect(url.domain(), &scid, local_addr, peer_addr, &mut config)
            .unwrap();

    println!(
        "connecting to {:} from {:} with scid {}",
        peer_addr,
        socket.local_addr().unwrap(),
        hex_dump(&scid)
    );
    info!(
        "connecting to {:} from {:} with scid {}",
        peer_addr,
        socket.local_addr().unwrap(),
        hex_dump(&scid)
    );

    let (write, send_info) = conn.send(&mut out).expect("initial send failed");

    while let Err(e) = socket.send_to(&out[..write], send_info.to) {
        if e.kind() == std::io::ErrorKind::WouldBlock {
            debug!("send() would block");
            continue;
        }

        panic!("send() failed: {:?}", e);
    }

    debug!("written {}", write);
    println!("written {}", write);

    let req_start = std::time::Instant::now();

    let mut req_sent = false;

    loop {
        poll.poll(&mut events, conn.timeout()).unwrap();

        // Read incoming UDP packets from the socket and feed them to quiche,
        // until there are no more packets to read.
        println!("\n==Read Loop====");
        'read: loop {
            // If the event loop reported no events, it means that the timeout
            // has expired, so handle it without attempting to read packets. We
            // will then proceed with the send loop.
            if events.is_empty() {
                debug!("timed out");
                println!(" - timed out : events empty");
                conn.on_timeout();
                break 'read;
            }

            let (len, from) = match socket.recv_from(&mut buf) {
                Ok(v) => v,

                Err(e) => {
                    // There are no more UDP packets to read, so end the read
                    // loop.
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("recv() would block");
                        println!(" - No more UDP packets to read");
                        break 'read;
                    }

                    panic!("recv() failed: {:?}", e);
                },
            };

            debug!("got {} bytes", len);
            println!(" - got {} bytes", len);

            let recv_info = quiche::RecvInfo {
                to: socket.local_addr().unwrap(),
                from,
            };

            // Process potentially coalesced packets.
            let read = match conn.recv(&mut buf[..len], recv_info) {
                Ok(v) => v,

                Err(e) => {
                    error!("recv failed: {:?}", e);
                    continue 'read;
                },
            };

            debug!("processed {} bytes", read);
            println!(" - processed {} bytes", read);
        }

        debug!("done reading");
        println!("===========");

        if conn.is_closed() {
            info!("connection closed, {:?}", conn.stats());
            println!("\n==Connection closed==, {:?}", conn.stats());
            break;
        }

        println!("== Send Stream ====");
        // Send an HTTP request as soon as the connection is established.
        if conn.is_established() && !req_sent {
            info!("sending HTTP request for {}", url.path());
            println!(" - sending HTTP request for {}", url.path());

            let req = format!("GET {}\r\n", url.path());
            conn.stream_send(HTTP_REQ_STREAM_ID, req.as_bytes(), true)
                .unwrap();

            req_sent = true;
        }
        println!("==Read stream====");
        // Process all readable streams.
        for s in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(s, &mut buf) {
                debug!("received {} bytes", read);
                println!(" - received {} bytes", read);

                let stream_buf = &buf[..read];

                debug!(
                    "stream {} has {} bytes (fin? {})",
                    s,
                    stream_buf.len(),
                    fin
                );
                println !(
                    "- stream {} has {} bytes (fin? {})",
                    s,
                    stream_buf.len(),
                    fin
                );

                print!("{}", unsafe {
                    std::str::from_utf8_unchecked(stream_buf)
                });

                // The server reported that it has no more data to send, which
                // we got the full response. Close the connection.
                if s == HTTP_REQ_STREAM_ID && fin {
                    info!(
                        "response received in {:?}, closing...",
                        req_start.elapsed()
                    );

                    conn.close(true, 0x00, b"kthxbye").unwrap();
                }
            }
        }

        // Generate outgoing QUIC packets and send them on the UDP socket, until
        // quiche reports that there are no more packets to be sent.
        println!("\n==Send Loop=====");
        loop {

            let (write, send_info) = match conn.send(&mut out) {
                Ok(v) => v,

                Err(quiche::Error::Done) => {
                    debug!("done writing");
                    println!(" - No more UDP packets to write");
                    break;
                },

                Err(e) => {
                    error!("send failed: {:?}", e);

                    conn.close(false, 0x1, b"fail").ok();
                    break;
                },
            };

            if let Err(e) = socket.send_to(&out[..write], send_info.to) {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    debug!("send() would block");
                    break;
                }

                panic!("send() failed: {:?}", e);
            }

            debug!("written {}", write);
            println!("- written {}", write);
        }
        println!("=========");
        if conn.is_closed() {
            info!("\n==Connection closed, {:?}", conn.stats());
            break;
        }
    }
}

fn hex_dump(buf: &[u8]) -> String {
    let vec: Vec<String> = buf.iter().map(|b| format!("{b:02x}")).collect();

    vec.join("")
}

// #[macro_use]
// extern crate log;
// extern crate common;
// mod prelude {
//     pub use std::env;
//     pub use std::collections::HashMap;
//     pub use std::time::{Duration, Instant};
//
//     pub use uuid::Uuid;
//     pub use ring::rand::*;
//
//     pub use quiche;
//     pub use mio::net::UdpSocket;
//     pub use common::*;
// }
//
// use quiche::{Connection, PathStats};
// use crate::prelude::*;
// const MSG_SIZE : usize = 100_000;
// const MAX_DATAGRAM_SIZE: usize = 1350;
//
// const HTTP_REQ_STREAM_ID: u64 = 4;
//
// fn main() {
//     // args [path, message_size]
//     // let args: Vec<String> = env::args().collect();
//     // let message_size : usize = args[1].parse().unwrap();
//     let mut message :[u8;MSG_SIZE] = [1;MSG_SIZE];
//     let mut records : HashMap<Uuid, Record> = HashMap::new();
//
//     let mut buf = [0; 65535];
//     let mut out = [0; MAX_DATAGRAM_SIZE];
//
//
//     // Setup the event loop.
//     let mut poll = mio::Poll::new().unwrap();
//     let mut events = mio::Events::with_capacity(1024);
//
//     // Create the UDP socket backing the QUIC connection, and register it withthe event loop.
//     let mut socket =
//         mio::net::UdpSocket::bind(CLIENT_ADDR.parse().unwrap()).unwrap();
//     poll.registry()
//         .register(&mut socket, mio::Token(0), mio::Interest::READABLE | mio::Interest::WRITABLE)
//         .unwrap();
//
//     let peer_addr = SERVER_ADDR.parse().unwrap();
//     let local_addr = socket.local_addr().unwrap();
//
//     // Create the configuration for the QUIC connection.
//     let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();
//
//     // *CAUTION*: this should not be set to `false` in production!!!
//     config.verify_peer(false);
//
//     config
//         .set_application_protos(&[
//             b"hq-interop",
//             b"hq-29",
//             b"hq-28",
//             b"hq-27",
//             b"http/0.9",
//         ])
//         .unwrap();
//
//     config.set_max_idle_timeout(5000);
//     config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
//     config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
//     config.set_initial_max_data(10_000_000);
//     config.set_initial_max_stream_data_bidi_local(1_000_000);
//     config.set_initial_max_stream_data_bidi_remote(1_000_000);
//     config.set_initial_max_streams_bidi(100);
//     config.set_initial_max_streams_uni(100);
//     config.set_disable_active_migration(true);
//     config.set_cc_algorithm_name("bbr2").unwrap(); // BBR2 is the CCA
//
//     // Generate a random source connection ID for the connection.
//     let mut scid = [0; quiche::MAX_CONN_ID_LEN];
//     SystemRandom::new().fill(&mut scid[..]).unwrap();
//
//     let scid = quiche::ConnectionId::from_ref(&scid);
//
//     // Get local address.
//     // let local_addr = CLIENT_ADDR.parse().unwrap();
//     let local_addr = socket.local_addr().unwrap();
//
//     // Create a QUIC connection and initiate handshake.
//     let mut conn = quiche::connect(Some("server_name"), &scid, local_addr, peer_addr, &mut config).unwrap();
//
//     println!(
//         "connecting to {:} from {:} with scid {}",
//         peer_addr,
//         socket.local_addr().unwrap(),
//         hex_dump(&scid)
//     );
//     info!(
//         "connecting to {:} from {:} with scid {}",
//         peer_addr,
//         socket.local_addr().unwrap(),
//         hex_dump(&scid)
//     );
//
//     let (write, send_info) = conn.send(&mut out).expect("initial send failed");
//
//     while let Err(e) = socket.send_to(&out[..write], send_info.to) {
//         if e.kind() == std::io::ErrorKind::WouldBlock {
//             println!("send() would block : Initial send");
//             debug!("send() would block");
//             continue;
//         }
//         println!("send() would block : Initial send panic");
//         panic!("send() failed: {:?}", e);
//     }
//     println!("Initial : written {}", write);
//     debug!("written {}", write);
//
//     let req_start = std::time::Instant::now();
//     let mut req_sent = false;
//
//     loop {
//         poll.poll(&mut events, conn.timeout()).unwrap();
//
//         // Read incoming UDP packets from the socket and feed them to quiche,
//         // until there are no more packets to read.
//         'read: loop {
//             println!("Reading packets");
//             // If the event loop reported no events, it means that the timeout
//             // has expired, so handle it without attempting to read packets. We
//             // will then proceed with the send loop.
//             if events.is_empty() {
//                 println!("events is empty : timeout");
//                 debug!("timed out");
//
//                 conn.on_timeout();
//                 break 'read;
//             }
//
//             let (len, from) = match socket.recv_from(&mut buf) {
//                 Ok(v) => v,
//
//                 Err(e) => {
//                     // There are no more UDP packets to read, so end the read
//                     // loop.
//                     if e.kind() == std::io::ErrorKind::WouldBlock {
//                         println!("recv() would block : socket");
//                         debug!("recv() would block");
//                         break 'read;
//                     }
//                     println!("recv() socket failed: {:?}", e);
//                     panic!("recv() failed: {:?}", e);
//                 },
//             };
//
//             debug!("got {} bytes from socket", len);
//             println!("got {} bytes from socket", len);
//
//             let recv_info = quiche::RecvInfo {
//                 to: socket.local_addr().unwrap(),
//                 from,
//             };
//
//             // Process potentially coalesced packets.
//             let read = match conn.recv(&mut buf[..len], recv_info) {
//                 Ok(v) => v,
//
//                 Err(e) => {
//                     error!("recv :conn failed: {:?}", e);
//                     println!("recv() : conn failed: {:?}", e);
//                     continue 'read;
//                 },
//             };
//
//             debug!("processed {} bytes from quiche", read);
//             println!("processed {} bytes from quiche", read);
//
//             // // Deserialize
//             // let ack_packet = PacketInfo::deserialize(&buf[..len]);
//             // measure_actual_rtt(ack_packet.id, &mut records);
//         }
//
//         debug!("done reading");
//         println!("done reading");
//
//         if conn.is_closed() {
//             info!("connection closed, {:?}", conn.stats());
//             println!("connection closed, {:?}", conn.stats());
//             break;
//         }
//
//         if conn.is_in_early_data() {
//             println!("conn is in early_data ");
//             info!("sending stream, message size : {}", MSG_SIZE);
//             println!("sending stream, message size : {}", MSG_SIZE);
//             // Write the message to the stream.
//             let stream_id = 0;
//             conn.stream_send(stream_id, &message, true).unwrap();
//
//             req_sent = true;
//         }
//         // Send the message as soon as the connection is established.
//         if conn.is_established() && !req_sent {
//             // let req = format!("GET {}\r\n", url.path());
//             // conn.stream_send(HTTP_REQ_STREAM_ID, req.as_bytes(), true)
//             //     .unwrap();
//
//             info!("sending stream, message size : {}", MSG_SIZE);
//             println!("sending stream, message size : {}", MSG_SIZE);
//             // Write the message to the stream.
//             let stream_id = 0;
//             conn.stream_send(stream_id, &message, true).unwrap();
//
//             req_sent = true;
//         }
//
//         // Process all readable streams.
//         for s in conn.readable() {
//             println!("Read stream from server, message size : {}", MSG_SIZE);
//             while let Ok((read, fin)) = conn.stream_recv(s, &mut buf) {
//                 debug!("received {} bytes", read);
//
//                 let stream_buf = &buf[..read];
//
//                 debug!(
//                     "stream {} has {} bytes (fin? {})",
//                     s,
//                     stream_buf.len(),
//                     fin
//                 );
//
//                 print!("{}", unsafe {
//                     std::str::from_utf8_unchecked(stream_buf)
//                 });
//
//                 // The server reported that it has no more data to send, which
//                 // we got the full response. Close the connection.
//                 if s == HTTP_REQ_STREAM_ID && fin {
//                     info!(
//                         "response received in {:?}, closing...",
//                         req_start.elapsed()
//                     );
//
//                     conn.close(true, 0x00, b"kthxbye").unwrap();
//                 }
//             }
//         }
//
//         // Generate outgoing QUIC packets and send them on the UDP socket, until quiche reports that there are no more packets to be sent.
//         loop {
//             println!("Send packets");
//             let (write, send_info) = match conn.send(&mut out) {
//                 Ok(v) => v,
//
//                 Err(quiche::Error::Done) => {
//                     debug!("done writing");
//                     println!("done writing");
//                     break;
//                 },
//
//                 Err(e) => {
//                     error!("send failed: {:?}", e);
//                     println!("send failed: {:?}", e);
//                     conn.close(false, 0x1, b"fail").ok();
//                     break;
//                 },
//             };
//
//             // // Create packet and serialize it.
//             // let packet_info = PacketInfo::new();
//             // let mut serialized : Vec<u8> = packet_info.serialize();
//             // out[..serialized.len()].copy_from_slice(&serialized);
//             //
//             // measure_path_stats_before_send(&conn, packet_info.id, &mut records);
//
//             if let Err(e) = socket.send_to(&out[..write], send_info.to) {
//                 if e.kind() == std::io::ErrorKind::WouldBlock {
//                     debug!("send() would block");
//                     break;
//                 }
//
//                 panic!("send() failed: {:?}", e);
//             }
//
//             debug!("written {}", write);
//             println!("written {}", write);
//         }
//
//         if conn.is_closed() {
//             info!("connection closed, {:?}", conn.stats());
//             break;
//         }
//     }
// }
//
// fn hex_dump(buf: &[u8]) -> String {
//     let vec: Vec<String> = buf.iter().map(|b| format!("{b:02x}")).collect();
//
//     vec.join("")
// }
//
// fn measure_path_stats_before_send(
//     conn : &Connection,
//     packet_id : Uuid,
//     records: &mut HashMap<Uuid, Record>
// ) {
//     // Assume that we have just one path between connection.
//
//     let path_stats : PathStats = conn.path_stats().next().unwrap();
//     // record
//     let record: Record = Record{
//         packet_id : packet_id,
//         actual_rtt : Duration::new(0,0),
//         send_timestamp : Instant::now(),
//         ack_timestamp : Instant::now(),
//         message_size : MSG_SIZE,
//         // path_stats : conn.path_stats(),
//         recv: path_stats.recv,
//         sent: path_stats.sent,
//         lost: path_stats.lost,
//         retrans: path_stats.retrans,
//         rtt: path_stats.rtt,
//         min_rtt: path_stats.min_rtt,
//         rttvar: path_stats.rttvar,
//         cwnd: path_stats.cwnd,
//         sent_bytes: path_stats.sent_bytes,
//         recv_bytes: path_stats.recv_bytes,
//         lost_bytes: path_stats.lost_bytes,
//         stream_retrans_bytes: path_stats.stream_retrans_bytes,
//         delivery_rate: path_stats.delivery_rate,
//     };
//     records.insert(packet_id, record);
// }
//
// fn measure_actual_rtt(
//     packet_id : Uuid,
//     records: &mut HashMap<Uuid, Record>
// ) {
//     // Calculate the actual rtt
//     let mut record: &mut Record = records.get_mut(&packet_id).unwrap();
//     record.set_ack_timestamp();
//     record.measure_actual_rtt();
//
//     debug!("Measure RTT : {}", record.actual_rtt.as_micros());
// }