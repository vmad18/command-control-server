use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use crate::lib::server::{Channel};
use crate::lib::util::{Node, Context};
use crate::lib::util::Packet;

pub struct WeatherParse {
    ctx: Arc<Mutex<Context>>
}

impl Node for WeatherParse {
    fn new(&self, dest: String, validate: bool, wait_time: u128, time: SystemTime, send_over: Channel) -> Box<dyn Node> {
        Box::new(WeatherParse{ ctx: Arc::new(Mutex::new(Context::new(dest, validate, wait_time, time, send_over))) })
    }
    
    fn gen_packets(&self) -> Vec<Packet> {
        todo!()
    }

    fn get_ctx(&mut self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }
    
}