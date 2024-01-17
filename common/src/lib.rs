use std::time::{Duration, Instant};
use std::fs::File;
use csv::Writer;

use serde_derive::{Deserialize, Serialize};
use serde_json;

use quiche::{PathStats};
use uuid::Uuid;
use plotters::prelude::*;
// use ndarray::prelude::*;
use plotters::element::ErrorBar;
// use ndarray::{array, Array, s};
pub const CLIENT_ADDR : &str = "127.0.0.1:3000";
pub const SERVER_ADDR : &str = "127.0.0.1:8000";
pub const MAX_DATAGRAM_SIZE: usize = 1350;

pub const UUID_SIZE : usize = 16;
/**
 Packet information
*/
pub struct PacketBody {
    pub id : Uuid,
    pub packet_size : usize,
}

impl PacketBody {
    pub fn new(packet_size : usize) -> PacketBody {
        return PacketBody{
            id: Uuid::new_v4(),
            packet_size : packet_size,
        }
    }
    //
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::from(self.id);
        serialized.extend(vec![1;self.packet_size]);

        return serialized;
    }

    pub fn deserialize(bytes : &[u8]) -> PacketBody {
        return PacketBody {
            id: Uuid::from_slice(&bytes[..UUID_SIZE]).unwrap(),
            packet_size : bytes.len()-UUID_SIZE,
        }
    }
}

pub struct Record {
    pub packet_id : Uuid,
    pub send_timestamp : Instant,
    pub ack_timestamp : Instant,
    pub actual_rtt : Duration,
    pub message_size : usize,
    //
    //pub path_stats : PathStats,
    // Path statistic
    // pub local_addr: SocketAddr,
    // pub peer_addr: SocketAddr,
    // pub validation_state: PathState,
    // pub active: bool,
    // pub recv: usize,
    // pub sent: usize,
    // pub lost: usize,
    // pub retrans: usize,
    // pub rtt: Duration,
    // pub min_rtt: Option<Duration>,
    // pub rttvar: Duration,
    // pub cwnd: usize,
    // pub sent_bytes: u64,
    // pub recv_bytes: u64,
    // pub lost_bytes: u64,
    // pub stream_retrans_bytes: u64,
    // pub pmtu: usize,
    // pub delivery_rate: u64,
}

// pub fn record_packets(data: Vec<Vec<str>>) {
//     let file = File::open("../../example.csv");
//
//     let mut writer = Writer::from_writer(file);
//
//     // Data
//     for row in data {
//         writer.write_record(&row).unwrap();
//         writer.flush().unwrap();
//     }
//     // writer.write_record(&[])
// }

pub fn draw(){
    // drawing area
    let drawing_area = BitMapBackend::new("../../images/2.1.png", (600,400)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();

    // chart
    let mut chart = ChartBuilder::on(&drawing_area)
        .caption("Title", ("Arial", 30))                    // title
        .set_left_and_bottom_label_area_size(40)            // Y, X axis size
        .build_cartesian_2d(0..100, 0..100)           // cartesian 2D
        .unwrap();

    // data 2차원 데이터

    // // draw
    // chart.draw_series(
    //     // data2d.iter()
    // ).unwrap();
    // evcxr_figure((640,240), |root| {
    //     _ = root.fill(&WHITE);
    //
    //
    //
    //     let x_axis = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    //
    //     chart.draw_series(LineSeries::new(
    //         x_axis.map(|x| (x,x)),
    //         &RED,
    //     ))?;
    // }).style("width:100%")
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let packet_info = PacketBody {
            id : Uuid::new_v4(),
            packet_size : MAX_DATAGRAM_SIZE-UUID_SIZE,
        };
        let serialized = packet_info.serialize();
        let buf = serialized.as_slice();

        let deserialized = PacketBody::deserialize(buf);

        assert_eq!(Uuid::from_slice(&buf[..UUID_SIZE]).unwrap(), deserialized.id);
    }
}
