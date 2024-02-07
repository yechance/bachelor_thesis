use std::time::{Duration, Instant};
use std::fs::{OpenOptions};
use csv::Writer;
pub use std::collections::HashMap;

use plotters::prelude::*;
pub const MAX_DATAGRAM_SIZE: usize = 1350;
pub const MAX_MSG_SIZE : usize = 1_000_000;
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
    // pub recv: usize,
    // pub sent: usize,
    // pub lost: usize,
    // pub retrans: usize,
    pub rtt: Duration,
    pub min_rtt: Option<Duration>,
    pub rttvar: Duration,
    pub cwnd: usize,
    // pub sent_bytes: u64,
    // pub recv_bytes: u64,
    // pub lost_bytes: u64,
    // pub stream_retrans_bytes: u64,
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
        // recv: path_stats.recv,
        // sent: path_stats.sent,
        // lost: path_stats.lost,
        // retrans: path_stats.retrans,
        rtt: path_stats.rtt,
        min_rtt: path_stats.min_rtt,
        rttvar: path_stats.rttvar,
        cwnd: path_stats.cwnd,
        // sent_bytes: path_stats.sent_bytes,
        // recv_bytes: path_stats.recv_bytes,
        // lost_bytes: path_stats.lost_bytes,
        // stream_retrans_bytes: path_stats.stream_retrans_bytes,
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
            // record.recv.to_string(),
            // record.sent.to_string(),
            // record.lost.to_string(),
            // record.retrans.to_string(),
            // format_duration(record.rtt),
            record.rtt.as_micros().to_string(),
            record.min_rtt.map(|rtt| rtt.as_micros().to_string()).unwrap(),
            // format_duration(record.rttvar),
            record.rttvar.as_micros().to_string(),
            record.cwnd.to_string(),
            // record.sent_bytes.to_string(),
            // record.recv_bytes.to_string(),
            // record.lost_bytes.to_string(),
            // record.stream_retrans_bytes.to_string(),
            record.delivery_rate.to_string(),
        ]).expect("write record error");
        writer.flush().expect("flush error");
    }
}

#[cfg(test)]
mod tests {
}
