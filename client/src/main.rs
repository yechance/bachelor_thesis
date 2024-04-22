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
mod prelude {
    pub use std::env;
    pub use std::fs::{OpenOptions};
    pub use std::collections::{HashMap, HashSet};
    pub use std::time::{Duration, Instant};
    pub use ring::rand::*;
    pub use rand::Rng;
    pub use quiche;
    pub use mio::net::UdpSocket;
    pub use csv::Writer;
}
use crate::prelude::*;

pub struct MessageGenerator {
    pub min_size : usize,
    pub max_size : usize,
}
impl MessageGenerator {
    pub fn generate_messages(&self, messages : &mut Vec<usize>, step : usize, step_mul : bool, repeat : usize) {
        let mut size : usize = self.min_size;
        while size <= self.max_size {
            for i in 0..repeat {
                messages.push(size);
            }
            if(step_mul){
                size *= step;
            } else {
                size += step;
            }
        }
    }
    pub fn generate_random_messages(&self, messages : &mut Vec<usize>, num_msg : usize) {
        let mut rng = rand::thread_rng();

        for _ in 0..num_msg {
            let random_number: usize = rng.gen_range(self.min_size..self.max_size);
            messages.push(random_number);
        }
    }

}
pub struct Record {
    pub message_size : usize,
    pub sending_rate: u64,
    pub rtt: Duration,
    pub latency : Duration,
    pub send_timestamp : Instant,
    pub ack_timestamp : Instant,
}
impl Record {
    pub fn set_ack_timestamp(&mut self){
        self.ack_timestamp = Instant::now();
    }
    pub fn measure_actual_rtt(&mut self){
        self.latency = self.ack_timestamp.duration_since(self.send_timestamp);
    }
}

pub fn measure_path_stats_before_send(
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
        sending_rate: path_stats.delivery_rate,
        rtt: path_stats.rtt,
        latency : Duration::new(0,0),
        send_timestamp : Instant::now(),
        ack_timestamp : Instant::now(),
    };
    records.insert(record_id, record);
}

pub fn measure_actual_latency(
    record_id : usize,
    records: &mut HashMap<usize, Record>
) {
    // Calculate the actual rtt
    let mut record: &mut Record = records.get_mut(&record_id).unwrap();
    record.set_ack_timestamp();
    record.measure_actual_rtt();
}
pub fn write_records_to_csv(filepath : &str, records : & HashMap<usize, Record>) {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filepath).unwrap();

    let mut writer = Writer::from_writer(file);

    // Data
    for (id, record) in records.iter() {
        writer.write_record(&[
            id.to_string(),
            record.message_size.to_string(),
            record.sending_rate.to_string(),
            record.rtt.as_micros().to_string(),
            record.latency.as_micros().to_string(),
        ]).expect("write record error");
        writer.flush().expect("flush error");
    }
}

pub const EXAMPLE_CSV : &str= "../plotting/example.csv";
pub const EXAMPLE_2_CSV : &str= "../plotting/example2.csv";
pub const EXAMPLE_3_CSV : &str= "../plotting/example3.csv";
pub const EXAMPLE_4_CSV : &str= "../plotting/example4.csv";
pub const ETHERNET_CSV : &str= "ethernet.csv";
pub const WIFI_CSV : &str= "wifi.csv";
const FIRST_STREAM_ID_UNI: u64 = 2;
pub const MAX_DATAGRAM_SIZE: usize = 1350;
pub const MAX_MSG_SIZE : usize = 6_000_000;
const K :usize = 1_000;
const M :usize = 1_000_000;
const NUM_MSG : usize = 5200;
fn main() {
    let mut buf = [0; 65535];
    let mut out = [0; MAX_DATAGRAM_SIZE];
    let message = vec![1;MAX_MSG_SIZE];

    // let filepath : &str  = "/test.csv";
    let filepath : &str  = "/app/data/ethernet_01loss.csv";
    // let filepath : &str  = "/app/data/wlan_100mbit_5ms.csv";

    // generate messages
    let mut messages : Vec<usize> = Vec::new();
    let message_generator : MessageGenerator = MessageGenerator{
        min_size : 500*K,
        max_size : 6*M,
    };
    message_generator.generate_random_messages(&mut messages, NUM_MSG);
    // record
    let mut records : HashMap<usize, Record> = HashMap::new();

    /** socket binding */
    // let mut url = url::Url::parse("https://server:4433/").unwrap();
    let mut url = url::Url::parse("https://127.0.0.1:4433/").unwrap();

    // Set up the event loop.
    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(1024);

    let peer_addr = url.socket_addrs(|| None).unwrap()[0];
    let bind_addr = match peer_addr {
        std::net::SocketAddr::V4(_) => "0.0.0.0:3344",
        std::net::SocketAddr::V6(_) => "[::]:0",
    };

    let mut socket =
        UdpSocket::bind(bind_addr.parse().unwrap()).unwrap();
    poll.registry()
        .register(&mut socket, mio::Token(0), mio::Interest::READABLE | mio::Interest::WRITABLE)
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
    config.set_initial_max_stream_data_uni(6_000_000); // 1M
    config.set_initial_max_streams_uni(100);
    config.set_disable_active_migration(true);
    config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR2);
    // config.set_cc_algorithm(quiche::CongestionControlAlgorithm::CUBIC);

    /** Connection */
    // Generate a random source connection ID for the connection.
    let mut scid_base = [0; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new().fill(&mut scid_base[..]).unwrap();
    let mut scid = quiche::ConnectionId::from_ref(&scid_base);

    let local_addr = socket.local_addr().unwrap();

    let mut conn = quiche::connect(url.domain(), &scid, local_addr, peer_addr, &mut config).unwrap();

    let mut inital_packet = true;
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
    let mut msg_idx: usize = 0; // message index
    let mut msg_size = messages[msg_idx]; // the current message size
    let mut rest_write = msg_size; // the rest bytes of the current message to be sent

    let mut stream_id : u64 = FIRST_STREAM_ID_UNI; // stream id
    let mut ready :bool = true;

    loop {
        poll.poll(&mut events, conn.timeout()).unwrap();

        /** Read packets */
        'read: loop {
            if events.is_empty() {
                debug!("timed out");
                // println!("\t timed out");
                conn.on_timeout();
                break 'read;
            }

            let (len, from) = match socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(e) => {
                    // There are no more UDP packets to read, so end the read loop.
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("recv() would block");
                        // This packet is Ack for a stream.
                        if !inital_packet {
                            if rest_write <= 0 && msg_idx < num_msg {
                                // record RTT
                                measure_actual_latency(msg_idx, &mut records);

                                // move next message
                                println!("[Sent Message idx : {}] ", msg_idx);
                                msg_idx += 1; // next message
                                ready = true;
                            }
                        } else {
                            inital_packet = false;
                        }
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
        /** Send messages via streams in the order */
        if conn.is_established() && msg_idx < num_msg
        {
            if ready {
                msg_size = messages[msg_idx]; // the current message size
                rest_write = msg_size; // the rest bytes of the current message to be sent
                stream_id = FIRST_STREAM_ID_UNI;
                ready = false;
                // measure
                measure_path_stats_before_send(&mut conn, &mut records, msg_idx, msg_size);
            }

            if rest_write > 0 {
                let next_stream = conn.stream_writable_next();
                let mut write_cap =0;

                if next_stream.is_some() {
                    stream_id= next_stream.unwrap();
                }
                if conn.stream_finished(stream_id) {
                    // println!("\t stream finished");
                } else {
                    let cap = conn.stream_capacity(stream_id).unwrap();
                    write_cap = std::cmp::min(rest_write, cap);
                }
                let written = match conn.stream_send(stream_id, &message[..rest_write], false) {
                    Ok(v) => v,
                    Err(quiche::Error::Done) => 0,
                    Err(e) => {
                        error!("{} stream send failed {:?}", conn.trace_id(), e);
                        println!("\t {} stream send failed {:?}", conn.trace_id(), e);
                        break;
                    },
                };
                rest_write -= written;
                println!("[Stream {}]\n \t Send Message idx : {}, rest bytes {}, written {}] ", stream_id, msg_idx, rest_write, written);
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
            info!("connection closed, {:?}", conn.stats());
            println!("\t connection closed, {:?}", conn.stats());
            break;
        }
    }
    // write the records to the csv file
    write_records_to_csv(filepath, &records);
}

