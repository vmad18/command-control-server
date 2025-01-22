use crate::lib::server::Channel;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct Context {
    pub dest: (String, String, String),
    pub validate: bool,
    wait_time: u128,
    time: SystemTime,
    pub send_over: Channel,
}

pub struct Packet {
    pub data: [u8; 56],
    pub size: usize,
}

pub trait Node: Send + Sync {
    fn gen_packets(&self) -> Vec<Packet>;

    fn get_ctx(&mut self) -> Arc<Mutex<Context>>;
}

impl Context {
    pub fn new(
        dest: (String, String, String),
        validate: bool,
        wait_time: u128,
        time: SystemTime,
        send_over: Channel,
    ) -> Self {
        Context {
            dest,
            validate,
            wait_time,
            time,
            send_over,
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
