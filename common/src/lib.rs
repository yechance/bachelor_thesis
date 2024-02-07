use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use csv::Writer;
use std::path::Path;
pub use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use serde_json;

use quiche::{PathStats};
use uuid::Uuid;
use plotters::prelude::*;
// use ndarray::prelude::*;
use plotters::element::ErrorBar;
// use ndarray::{array, Array, s};
pub const CLIENT_ADDR : &str = "127.0.0.1:3300";
pub const SERVER_ADDR : &str = "127.0.0.1:8080";
pub const MAX_DATAGRAM_SIZE: usize = 1350;
pub const UUID_SIZE : usize = 16;
pub const DATAGRAM_SIZE : usize = 1183;

pub const EXAMPLE_CSV : &str= "../plotting/example.csv";
pub const EXAMPLE_2_CSV : &str= "../plotting/example2.csv";
pub const EXAMPLE_3_CSV : &str= "../plotting/example3.csv";
pub const EXAMPLE_4_CSV : &str= "../plotting/example4.csv";
pub const ETHERNET_CSV : &str= "ethernet.csv";
pub const WIFI_CSV : &str= "wifi.csv";


pub struct MessageGenerator {
    pub min_size : usize,
    pub max_size : usize,
    pub step : usize,
    pub step_mul : bool,
    pub repeat : usize,
    // pub random : bool,
    // pub message_num : usize,
}

impl MessageGenerator {
    pub fn generate_messages(&self, messages : &mut Vec<usize>) {
        let mut size : usize = self.min_size;
        while size <= self.max_size {
            for i in 0..self.repeat {
                messages.push(size);
            }
            if(self.step_mul){
                size *= self.step;
            } else {
                size += self.step;
            }
        }
    }
}

pub struct Record {
    pub message_size : usize,
    pub send_timestamp : Instant,
    pub ack_timestamp : Instant,
    pub actual_rtt : Duration,
    pub recv: usize,
    pub sent: usize,
    pub lost: usize,
    pub retrans: usize,
    pub rtt: Duration,
    pub min_rtt: Option<Duration>,
    pub rttvar: Duration,
    pub cwnd: usize,
    pub sent_bytes: u64,
    pub recv_bytes: u64,
    pub lost_bytes: u64,
    pub stream_retrans_bytes: u64,
    pub delivery_rate: u64,
}

impl Record {
    pub fn set_ack_timestamp(&mut self){
        self.ack_timestamp = Instant::now();
    }
    pub fn measure_actual_rtt(&mut self){
        self.actual_rtt = self.ack_timestamp.duration_since(self.send_timestamp);
    }

}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let millis = duration.subsec_millis();

    format!("{:02}:{:02}.{:03}", seconds / 60, seconds % 60, millis)
}
pub fn write_records_to_csv(filepath : &str, records : & HashMap<usize, Record>) {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filepath).unwrap();

    let mut writer = Writer::from_writer(file);

    // Header
    // writer.write_record(&[
    //     "message_size",
    //     "actual rtt",
    //     "recv",
    //     "sent",
    //     "lost",
    //     "retrans",
    //     "rtt",
    //     "min_rtt",
    //     "rttvar",
    //     "cwnd",
    //     "sent_bytes",
    //     "recv bytes",
    //     "lost_bytes",
    //     "stream_retrans_bytes",
    //     "delivery_rate"]).unwrap();

    // Data
    for (id, record) in records.iter() {
        writer.write_record(&[
            record.message_size.to_string(),
            // format_duration(record.actual_rtt),
            record.actual_rtt.as_micros().to_string(),
            record.recv.to_string(),
            record.sent.to_string(),
            record.lost.to_string(),
            record.retrans.to_string(),
            // format_duration(record.rtt),
            record.rtt.as_micros().to_string(),
            record.min_rtt.map(|rtt| rtt.as_micros().to_string()).unwrap(),
            // format_duration(record.rttvar),
            record.rttvar.as_micros().to_string(),
            record.cwnd.to_string(),
            record.sent_bytes.to_string(),
            record.recv_bytes.to_string(),
            record.lost_bytes.to_string(),
            record.stream_retrans_bytes.to_string(),
            record.delivery_rate.to_string(),
        ]).expect("write record error");
        writer.flush().expect("flush error");
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use super::*;

    #[test]
    fn test_write_records_to_csv(){
        let mut records : HashMap<usize, Record> = HashMap::new();
        let r1 : Record = Record {
            message_size : 0,
            actual_rtt : Duration::new(5,0),
            send_timestamp : Instant::now(),
            ack_timestamp : Instant::now(),
            recv: 0,
            sent: 0,
            lost: 0,
            retrans: 0,
            rtt: Duration::new(5,0),
            min_rtt: Some(Duration::new(5,0)),
            rttvar: Duration::new(5,0),
            cwnd: 0,
            sent_bytes: 0,
            recv_bytes: 0,
            lost_bytes: 0,
            stream_retrans_bytes: 0,
            delivery_rate: 0,
        };
        let r2 : Record = Record {
            actual_rtt : Duration::new(10,0),
            send_timestamp : Instant::now(),
            ack_timestamp : Instant::now(),
            message_size : 200,
            recv: 0,
            sent: 0,
            lost: 0,
            retrans: 0,
            rtt: Duration::new(5,0),
            min_rtt: Some(Duration::new(5,0)),
            rttvar: Duration::new(5,0),
            cwnd: 0,
            sent_bytes: 0,
            recv_bytes: 0,
            lost_bytes: 0,
            stream_retrans_bytes: 0,
            delivery_rate: 0,
        };
        records.insert(0, r1);
        records.insert(1, r2);

        write_records_to_csv(EXAMPLE_CSV, &records);
    }
}
