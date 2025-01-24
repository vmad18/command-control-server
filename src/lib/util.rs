use crate::lib::server::Channel;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub const PACKET_SIZE: usize = 128;

pub struct Context {
    pub dest: (String, String, String),
    pub validate: bool,
    wait_time: u128,
    time: SystemTime,
    pub send_over: Channel,
    pub finished: bool
}

pub struct Packet {
    pub data: [u8; PACKET_SIZE],
    pub size: usize,
}

impl Packet {

    pub fn insert(&mut self, new_data: &[u8], len: usize) {
        for i in 0..len {
            self.data[i + self.size] = new_data[i];
        }
        self.size += len;
    }

}

pub trait Node: Send + Sync {
    fn gen_packets(&mut self) -> Vec<Packet>;

    fn get_ctx(&mut self) -> Arc<Mutex<Context>>;
}

pub fn str_2_bytes(str: &String) -> Vec<u8> {
    str.as_bytes().to_vec()
}

impl Context {
    pub fn new(
        dest: (String, String, String),
        validate: bool,
        wait_time: u128,
        time: SystemTime,
        send_over: Channel,
        finished: bool
    ) -> Self {
        Context {
            dest,
            validate,
            wait_time,
            time,
            send_over,
            finished
        }
    }

    pub fn reached_time(&mut self) -> bool {
        if self.time.elapsed().unwrap().as_millis() >= self.wait_time {
            self.time = SystemTime::now();
            true
        } else {
            false
        }
    }
}
