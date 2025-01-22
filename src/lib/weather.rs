use crate::lib::server::Channel;
use crate::lib::util::Packet;
use crate::lib::util::{Context, Node};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct WeatherParse {
    ctx: Arc<Mutex<Context>>,
}

impl WeatherParse {
    pub fn new(
        dest: (String, String, String),
        validate: bool,
        wait_time: u128,
        time: SystemTime,
        send_over: Channel,
    ) -> Box<dyn Node> {
        Box::new(WeatherParse {
            ctx: Arc::new(Mutex::new(Context::new(
                dest, validate, wait_time, time, send_over,
            ))),
        })
    }
}

impl Node for WeatherParse {
    fn gen_packets(&self) -> Vec<Packet> {
        let data = "Hello how are you?";
        let data_bytes = data.as_bytes();

        let mut packet_data: [u8; 56] = [0; 56];

        for i in 0..data_bytes.len() {
            packet_data[i] = data_bytes[i];
        }

        let packet = Packet {
            data: packet_data,
            size: data_bytes.len(),
        };

        let packets = vec![packet];
        packets
    }

    fn get_ctx(&mut self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }
}
