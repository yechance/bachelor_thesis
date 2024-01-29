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
extern crate common;

mod prelude {
    pub use std::env;
    pub use std::collections::HashMap;
    pub use std::time::{Duration, Instant};

    pub use uuid::Uuid;
    pub use ring::rand::*;

    pub use quiche;
    pub use mio::net::UdpSocket;
    pub use common::*;
}

use std::hash::Hash;
use crate::prelude::*;

const HTTP_REQ_STREAM_ID: u64 = 4;

const MSG_SIZE : usize = 1_000_000;

fn main() {
    let mut buf = [0; 65535];
    let mut out = [0; MAX_DATAGRAM_SIZE];

    // let args: Vec<String> = env::args().collect();
    // let message_size : usize = args[1].parse().unwrap();
    let filepath : &str  = EXAMPLE_CSV;

    let mut messages : Vec<Vec<u8>> = Vec::new();
    let message_generator : MessageGenerator = MessageGenerator{
        min_size : 10_000,
        max_size : 15_000,
        step : 100,
        repeat : 10,
    };
    message_generator.generate_messages(&mut messages);

    let mut streams_sent : Vec<bool> = vec![false;messages.len()];
    let mut records : HashMap<usize, Record> = HashMap::new();

    let mut url = url::Url::parse("https://127.0.0.1:4433/").unwrap();

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
        UdpSocket::bind(bind_addr.parse().unwrap()).unwrap();
    poll.registry()
        .register(&mut socket, mio::Token(0), mio::Interest::READABLE)
        .unwrap();

    // Create the configuration for the QUIC connection.
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();

    // *CAUTION*: this should not be set to `false` in production!!!
    config.verify_peer(false);

    config
        .set_application_protos(&[b"http/0.9", ])
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
    config
        .set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR2);

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

    let num_msg: usize = messages.len();
    let mut idx : usize= 0;

    loop {
        poll.poll(&mut events, conn.timeout()).unwrap();

        // Read incoming UDP packets from the socket and feed them to quiche,
        // until there are no more packets to read.
        'read: loop {
            // If the event loop reported no events, it means that the timeout
            // has expired, so handle it without attempting to read packets. We
            // will then proceed with the send loop.
            if events.is_empty() {
                debug!("timed out");
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
                        break 'read;
                    }

                    panic!("recv() failed: {:?}", e);
                },
            };

            debug!("got {} bytes", len);

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
        }

        if conn.is_closed() {
            info!("connection closed, {:?}", conn.stats());
            break;
        }

        // Process all readable streams.
        for s in conn.readable() {
            println!("[Read stream]");
            while let Ok((read, fin)) = conn.stream_recv(s, &mut buf) {
                debug!("received {} bytes", read);

                let stream_buf = &buf[..read];
                debug!(
                    "stream {} has {} bytes (fin? {})",
                    s,
                    stream_buf.len(),
                    fin
                );
                println !(
                    " - stream {} has {} bytes (fin? {})",
                    s,
                    stream_buf.len(),
                    fin
                );

                print!("{}", unsafe {
                    std::str::from_utf8_unchecked(stream_buf)
                });

                // The server reported that it has no more data to send, which
                // we got the full response. Close the connection.
                if fin {
                    let message_idx : usize = (s/4) as usize;
                    measure_actual_rtt(message_idx, &mut records);
                    streams_sent[message_idx] = true;
                    println!(" - Ack stream id : {}", s as usize);

                    if idx == num_msg {
                        println!("connection close");
                        conn.close(true, 0x00, b"kthxbye").unwrap();
                    }
                }
            }
            println!("[End Read Stream]");
        }

        // Send an HTTP request as soon as the connection is established.
        if conn.is_established() && idx < num_msg &&
            (idx == 0 || (idx > 0 && streams_sent[idx-1]))
        {
            println!("[Send Stream]");
            measure_path_stats_before_send(&conn, &mut records, idx, messages[idx].len());

            let written = match conn.stream_send((idx*4) as u64, &messages[idx], true) {
                Ok(v) => v,
                Err(quiche::Error::Done) => 0,

                Err(e) => {
                    error!("{} stream send failed {:?}", conn.trace_id(), e);
                    return;
                },
            };

            println !(
                "- Send stream {} has {} bytes, {} bytes written",
                idx*4,
                messages[idx].len(),
                written,
            );

            idx += 1;
        }

        // Generate outgoing QUIC packets and send them on the UDP socket, until
        // quiche reports that there are no more packets to be sent.
        loop {
            let (write, send_info) = match conn.send(&mut out) {
                Ok(v) => v,

                Err(quiche::Error::Done) => {
                    debug!("done writing");
                    break;
                },

                Err(e) => {
                    error!("send failed: {:?}", e);

                    conn.close(false, 0x1, b"fail").ok();
                    break;
                },
            };

            // measure_path_stats_before_send(&conn, packet_info.id, &mut records);
            if let Err(e) = socket.send_to(&out[..write], send_info.to) {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    debug!("send() would block");
                    break;
                }

                panic!("send() failed: {:?}", e);
            }

            debug!("written {}", write);
        }
        if conn.is_closed() {
            info!("\n==Connection closed, {:?}", conn.stats());
            break;
        }
    }

    // for (idx, record) in records {
    //     println!("record id : {}", idx);
    //     println!("message size : {}", record.message_size);
    // }

    // write the records to the csv file
    write_records_to_csv(filepath, &records);
}

fn hex_dump(buf: &[u8]) -> String {
    let vec: Vec<String> = buf.iter().map(|b| format!("{b:02x}")).collect();

    vec.join("")
}

fn measure_path_stats_before_send(
    conn : &quiche::Connection,
    records: &mut HashMap<usize, Record>,
    record_id : usize,
    message_size : usize,
) {
    // Assume that we have just one path between connection.

    let path_stats : quiche::PathStats = conn.path_stats().next().unwrap();
    // record
    let record: Record = Record{
        message_size : message_size,
        actual_rtt : Duration::new(0,0),
        send_timestamp : Instant::now(),
        ack_timestamp : Instant::now(),
        // path_stats : conn.path_stats(),
        recv: path_stats.recv,
        sent: path_stats.sent,
        lost: path_stats.lost,
        retrans: path_stats.retrans,
        rtt: path_stats.rtt,
        min_rtt: path_stats.min_rtt,
        rttvar: path_stats.rttvar,
        cwnd: path_stats.cwnd,
        sent_bytes: path_stats.sent_bytes,
        recv_bytes: path_stats.recv_bytes,
        lost_bytes: path_stats.lost_bytes,
        stream_retrans_bytes: path_stats.stream_retrans_bytes,
        delivery_rate: path_stats.delivery_rate,
    };
    records.insert(record_id, record);
}

fn measure_actual_rtt(
    record_id : usize,
    records: &mut HashMap<usize, Record>
) {
    // Calculate the actual rtt
    let mut record: &mut Record = records.get_mut(&record_id).unwrap();
    record.set_ack_timestamp();
    record.measure_actual_rtt();

    debug!("Measure RTT : {}", record.actual_rtt.as_micros());
}