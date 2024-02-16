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
    pub use std::collections::{HashMap, HashSet};
    pub use std::time::{Duration, Instant};
    pub use ring::rand::*;
    pub use quiche;
    pub use mio::net::UdpSocket;
    pub use common::*;
}
use std::hash::Hash;
use crate::prelude::*;
const HTTP_REQ_STREAM_ID: u64 = 4;


fn main() {
    let mut buf = [0; 65535];
    let mut out = [0; MAX_DATAGRAM_SIZE];
    let message = vec![1;MAX_MSG_SIZE];

    // let args: Vec<String> = env::args().collect();
    // let message_size : usize = args[1].parse().unwrap();
    let filepath : &str  = EXAMPLE_3_CSV;

    // generate messages
    let mut messages : Vec<usize> = Vec::new();
    let message_generator : MessageGenerator = MessageGenerator{
        min_size : 1_000_000,
        max_size : MAX_MSG_SIZE,
        step : 10,
        step_mul : true,
        repeat : 1000,
    };
    message_generator.generate_messages(&mut messages);
    // record
    let mut records : HashMap<usize, Record> = HashMap::new();

    /** socket binding */
    let mut url = url::Url::parse("https://127.0.0.1:4433/").unwrap();

    // Set up the event loop.
    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(1024);

    let peer_addr = url.socket_addrs(|| None).unwrap()[0];
    let bind_addr = match peer_addr {
        std::net::SocketAddr::V4(_) => "127.0.0.1:3344",
        std::net::SocketAddr::V6(_) => "[::]:0",
    };

    let mut socket =
        UdpSocket::bind(bind_addr.parse().unwrap()).unwrap();
    poll.registry()
        .register(&mut socket, mio::Token(0), mio::Interest::READABLE)
        .unwrap();

    /** Configuration of quiche */
    // Create the configuration for the QUIC connection.
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();

    // *CAUTION*: this should not be set to `false` in production!!!
    config.verify_peer(false);

    config
        .set_application_protos(&[b"http/0.9", ])
        .unwrap();

    config.set_max_idle_timeout(5_000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(10_000_000);
    config.set_initial_max_stream_data_bidi_remote(10_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_initial_max_streams_uni(100);
    config.set_disable_active_migration(true);
    config
        .set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR2);

    /** Connection */
    // Generate a random source connection ID for the connection.
    let mut scid_base = [0; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new().fill(&mut scid_base[..]).unwrap();
    let mut scid = quiche::ConnectionId::from_ref(&scid_base);

    let local_addr = socket.local_addr().unwrap();

    let mut conn = quiche::connect(url.domain(), &scid, local_addr, peer_addr, &mut config).unwrap();

    /** Initial send for the server to accept this client */
    let (write, send_info) = conn.send(&mut out).expect("initial send failed");

    while let Err(e) = socket.send_to(&out[..write], send_info.to) {
        if e.kind() == std::io::ErrorKind::WouldBlock {
            debug!("send() would block");
            continue;
        }

        panic!("send() failed: {:?}", e);
    }

    /** variables for sending streams */
    let num_msg: usize = messages.len(); // the number of messages
    let mut idx : usize= 0; // message index

    let mut stream_id : u64 = 0; // stream id
    let mut message_size = messages[idx]; // the current message size
    let mut total_written = 0; // the total written bytes of the current message
    let mut rest_write = message_size - total_written; // the rest bytes of the current message to be sent

    let mut msg_send_started = false; // true if sending the current message starts
    let mut send_timestamp;

    // let mut path_stats;
    // measure_path_stats_before_send(&conn, &mut records, idx, messages[idx]);


    let mut timeout_time = Instant::now() + conn.timeout().unwrap();

    loop {
        // poll.poll(&mut events, conn.timeout()).unwrap();
        poll.poll(&mut events, Some(Duration::from_micros(100))).unwrap();

        /** Send messages via streams in the order */
        if conn.is_established() && idx < num_msg
        {
            // if it's the first time to send the streams for a message, record the time and statistics
            if !msg_send_started {
                // Initiate the info for sending a message.
                // stream_id = 0;
                message_size = messages[idx];
                total_written = 0;
                rest_write = message_size - total_written;

                send_timestamp = Instant::now();
                // path_stats = conn.path_stats();

                msg_send_started = true;
            }

            // If there are the rest bytes to be sent for a message, send them again.
            if rest_write > 0 {
                let written = match conn.stream_send(stream_id, &message[..rest_write], true) {
                    Ok(v) => v,
                    Err(quiche::Error::Done) => 0,
                    Err(e) => {
                        error!("{} stream send failed {:?}", conn.trace_id(), e);
                        println!("\t {} stream send failed {:?}", conn.trace_id(), e);
                        return;
                    },
                };

                total_written += written;
                rest_write = message_size - total_written;

                println!("[Stream {}]\n \t Send Message idx : {}, total sent bytes : {}, rest bytes {}, written {}] ", stream_id, idx, total_written, rest_write,written);
                // streams_in_use.insert(stream_id);
                stream_id += 4;
                stream_id %= 400;
            }
        }

        /** Send packets until there's no more packet to be sent */
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

        /** Checking if the connection is closed */
        if conn.is_closed() {
            info!("\n==Connection closed, {:?}", conn.stats());
            if idx < num_msg {
                /** reconnect */
                SystemRandom::new().fill(&mut scid_base[..]).unwrap();
                scid = quiche::ConnectionId::from_ref(&scid_base);

                conn = quiche::connect(url.domain(), &scid, local_addr, peer_addr, &mut config).unwrap();

                let (write, send_info) = conn.send(&mut out).expect("initial send failed");

                while let Err(e) = socket.send_to(&out[..write], send_info.to) {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("send() would block");
                        continue;
                    }

                    panic!("send() failed: {:?}", e);
                }
            } else {
                println!("\t connection closed, {:?}", conn.stats());
                break;
            }
        }

        /** Read stream */
        println!("[Read stream]");
        // Process all readable streams.
        for s in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(s, &mut buf) {
                debug!("received {} bytes", read);
                let stream_buf = &buf[..read];
                debug!(
                    "stream {} has {} bytes (fin? {})",
                    s,
                    stream_buf.len(),
                    fin
                );
                println!(
                    "\t stream {} has {} bytes (fin? {})",
                    s,
                    stream_buf.len(),
                    fin
                );

                print!("{}", unsafe {
                    std::str::from_utf8_unchecked(stream_buf)
                });

                if fin {
                    /** stream is finished */
                    println!("\t read {} stream finished", s);
                    // measure_actual_rtt(idx, &mut records);
                    // streams_in_use.remove(&s);

                    if rest_write <= 0 {
                        // /** Sending the current message is finished */
                        // it means that sending a message is done.
                        // record the measurement

                        // move to the next message
                        idx += 1;
                        msg_send_started = false;
                    }

                    if idx == num_msg {
                        println!("[Close connection : All stream read]");
                        conn.close(true, 0x00, b"kthxbye").unwrap();
                    }
                }
            }
        }
        /** Read packets */
        'read: loop {
            if events.is_empty() {
                debug!("timed out");
                println!("\t timed out");
                conn.on_timeout();
                break 'read;
            }

            let (len, from) = match socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(e) => {
                    // There are no more UDP packets to read, so end the read loop.
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("recv() would block");
                        println!("[end read]");
                        break 'read;
                    }
                    panic!("recv() failed: {:?}", e);
                },
            };

            debug!("got {} bytes", len);

            let recv_info = quiche::RecvInfo {to: socket.local_addr().unwrap(), from };

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
        /** Checking if the connection is closed */
        if conn.is_closed() {
            info!("connection closed, {:?}", conn.stats());
            if idx < num_msg {
                /** reconnect */
                SystemRandom::new().fill(&mut scid_base[..]).unwrap();
                scid = quiche::ConnectionId::from_ref(&scid_base);

                conn = quiche::connect(url.domain(), &scid, local_addr, peer_addr, &mut config).unwrap();

                let (write, send_info) = conn.send(&mut out).expect("initial send failed");

                while let Err(e) = socket.send_to(&out[..write], send_info.to) {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("send() would block");
                        continue;
                    }

                    panic!("send() failed: {:?}", e);
                }
            } else {
                println!("\t connection closed, {:?}", conn.stats());
                break;
            }

        }
    }
    // write the records to the csv file
    // write_records_to_csv(filepath, &records);
}

fn hex_dump(buf: &[u8]) -> String {
    let vec: Vec<String> = buf.iter().map(|b| format!("{b:02x}")).collect();

    vec.join("")
}



