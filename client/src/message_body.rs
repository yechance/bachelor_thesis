use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
struct MessageBody {
    pub req_send_timestamp : Instant,
    pub req_receive_timestamp : Instant,
    pub res_send_timestamp : Instant,
    pub res_receive_timestamp : Instant,
    pub body : Vec<u8>
}

impl MessageBody {

}