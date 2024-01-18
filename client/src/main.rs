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
use crate::prelude::*;

// #[derive(Serialize, Deserialize)]

fn main() {
    // args [path, message_size]
    // let args: Vec<String> = env::args().collect();
    // let message_size : usize = args[1].parse().unwrap();
    let message_size : usize = 100000;
    let mut sent_bytes = 0;
    let mut records : HashMap<Uuid, Record> = HashMap::new();

    // Buffer & Setup the event loop
    // 연결 할 서버의 주소를 받아오고 버퍼들을 준비한다.
    let peer_addr = SERVER_ADDR.parse().unwrap();
    let mut buf = [0;65535];
    let mut out = [0;MAX_DATAGRAM_SIZE];
    // 비동기 이벤트 루프를 만들기 위한 세팅
    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(1024);

    // Create the UDP socket backing the QUIC connection, and register it with the event loop.
    // 해당 서버 소켓을 poll에 등록을 한다.
    let mut socket = UdpSocket::bind(peer_addr).unwrap();
    poll.registry()
        .register(&mut socket, mio::Token(0), mio::Interest::READABLE | mio::Interest::WRITABLE)
        .unwrap();

    // Create the configuration for the QUIC connection.
    // QUICHE 설정
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();
    config.set_application_protos(&[
            b"hq-interop",
            b"hq-29",
            b"hq-28",
            b"hq-27",
            b"http/0.9",
            ]).unwrap();
    config.set_max_idle_timeout(5000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_initial_max_streams_uni(100);
    config.set_disable_active_migration(true);
    config.set_cc_algorithm_name("bbr2").unwrap(); // BBR2 is the CCA

    // Generate a random source connection ID for the connection.
    // 연결용 아이디 생성
    let mut scid = [0; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new().fill(&mut scid[..]).unwrap();
    let scid = quiche::ConnectionId::from_ref(&scid);

    // Get local address.
    let local_addr = CLIENT_ADDR.parse().unwrap();
    let server_name = "server";

    // Create a QUIC connection and initiate handshake.
    let mut conn = quiche::connect(Some(&server_name), &scid, local_addr, peer_addr, &mut config).unwrap();

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

    // 이벤트 루프 : 패킷 송수신
    loop {
        poll.poll(&mut events, conn.timeout()).unwrap();

        // Read incoming UDP packets from the socket and feed them to quiche,
        // until there are no more packets to read.
        'read: loop {
            // If the event loop reported no events, it means that the timeout has expired,
            // so handle it without attempting to read packets.
            // We will then proceed with the send loop.
            if events.is_empty() {
                debug!("timed out");

                conn.on_timeout();
                break 'read;
            }

            // 패킷 수신
            let (len, from) = match socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(e) => {
                    // There are no more UDP packets to read, so end the read loop.
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
            // // Deserialize
            // let ack_packet = PacketBody::deserialize(buf[..len]);
            //
            // // Calculate the actual rtt
            // let mut record = records.get(ack_packet.id).unwrap();
            // record.ack_timestamp = Instant::now();
            // record.actual_rtt = record.send_timestamp.elapsed();
            debug!("processed {} bytes", read);
        }

        debug!("done reading");

        if conn.is_closed() {
            info!("connection closed, {:?}", conn.stats());
            break;
        }
        {
        // // Send an HTTP request as soon as the connection is established.
        // if conn.is_established() && !req_sent {
        //     // info!("sending HTTP request for {}", url.path());
        //     //
        //     // let req = format!("GET {}\r\n", url.path());
        //     conn.stream_send(HTTP_REQ_STREAM_ID, req.as_bytes(), true)
        //         .unwrap();
        //
        //     req_sent = true;
        // }
        //
        // // Process all readable streams.
        // for s in conn.readable() {
        //     while let Ok((read, fin)) = conn.stream_recv(s, &mut buf) {
        //         debug!("received {} bytes", read);
        //
        //         let stream_buf = &buf[..read];
        //
        //         debug!(
        //             "stream {} has {} bytes (fin? {})",
        //             s,
        //             stream_buf.len(),
        //             fin
        //         );
        //
        //         print!("{}", unsafe {
        //             std::str::from_utf8_unchecked(stream_buf)
        //         });
        //
        //         // The server reported that it has no more data to send, which
        //         // we got the full response. Close the connection.
        //         if s == HTTP_REQ_STREAM_ID && fin {
        //             info!(
        //                 "response received in {:?}, closing...",
        //                 req_start.elapsed()
        //             );
        //
        //             conn.close(true, 0x00, b"kthxbye").unwrap();
        //         }
        //     }
        // }
        }

        // Generate outgoing QUIC packets and send them on the UDP socket,
        // until quiche reports that there are no more packets to be sent.
        'write: loop {
            // Calculate the size of the dgram packet
            if sent_bytes >= message_size {
                break 'write;
            }
            let mut packet_size = MAX_DATAGRAM_SIZE - UUID_SIZE;
            if sent_bytes + packet_size > message_size {
                packet_size = message_size - sent_bytes;
            }
            // Create packet and serialize it.
            let packet_body = PacketBody::new(packet_size);
            let mut serialized : Vec<u8> = packet_body.serialize();
            // record
            let mut record: Record = Record{
                packet_id : packet_body.id,
                actual_rtt : Duration::new(0,0),
                send_timestamp : Instant::now(),
                ack_timestamp : Instant::now(),
                message_size : message_size,
                // path_stats : conn.path_stats().clone(),
            };
            records.insert(record.packet_id, record);

            let (write, send_info) = match conn.send(serialized.as_mut_slice()) {
                Ok(v) => v,
                Err(quiche::Error::Done) => {
                    debug!("done writing");
                    break 'write;
                },
                Err(e) => {
                    error!("send failed: {:?}", e);

                    conn.close(false, 0x1, b"fail").ok();
                    break 'write;
                },
            };
            if let Err(e) = socket.send_to(&serialized[..write], send_info.to) {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    debug!("send() would block");
                    break;
                }

                panic!("send() failed: {:?}", e);
            }

            debug!("written {}", write);
        }

        if conn.is_closed() {
            info!("connection closed, {:?}", conn.stats());
            break;
        }
    }

    // Draw Data
    // or
    // Write the data
}

fn hex_dump(buf: &[u8]) -> String {
    let vec: Vec<String> = buf.iter().map(|b| format!("{b:02x}")).collect();

    vec.join("")
}