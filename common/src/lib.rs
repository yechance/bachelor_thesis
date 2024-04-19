use std::time::{Duration, Instant};
use std::fs::{OpenOptions};
use csv::Writer;
pub use std::collections::HashMap;
use rand::Rng;
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

#[cfg(test)]
mod tests {
}
