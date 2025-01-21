use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use crate::lib::server::{Channel};

pub struct Context {
    pub dest: String,
    validate: bool,
    wait_time: u128,
    time: SystemTime,
    send_over: Channel
}

pub struct Packet {
    pub data: [u8; 512]
}

pub trait Node: Send + Sync {
    fn new(&self, dest: String, validate: bool, wait_time: u128, time: SystemTime, send_over: Channel) -> Box<dyn Node>;

    fn gen_packets(&self) -> Vec<Packet>;

    fn get_ctx(&mut self) -> Arc<Mutex<Context>>;
}

impl Context {
    pub fn new(dest: String, validate: bool, wait_time: u128, time: SystemTime, send_over: Channel) -> Self {
        Context{ dest, validate, wait_time, time, send_over}
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