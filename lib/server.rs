use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::net::UdpSocket;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::lib::util::{Context, Node, Packet};

pub struct Service {
    packet: Packet,
    packet_id: u128,
}

pub struct CommandControl {
    nodes: Vec<Box<dyn Node>>
}

pub enum Channel {
    UDP,
    TCP,
    RF
}

impl CommandControl {

    pub fn create() -> Self {
        CommandControl {
            nodes: vec![]
        }
    }

    pub fn send(&self) {

    }

    pub fn push_node(&mut self, node: Box<dyn Node>) {
        self.nodes.push(node);
    }

    // pub fn validate_response(&self, received: Vec<Packet>) -> bool {
    //     // todo()
    // }

    fn visited<T: Eq + Hash>(map: &HashMap<T, (Arc<Service>, bool)>) -> bool {
        for key in map.keys() {
            if !(&map.get(key).unwrap().1) {
                return false;
            }
        }

        true
    }

    fn send_udp(mut services: HashMap<u128, (Arc<Service>, bool)>, ctx: Arc<Mutex<Context>>) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        while !Self::visited::<u128>(&services) {
            for id in services.clone().keys() {
                let (service, visited) = services.get(id).unwrap();
                if *visited { continue; }
                socket.send_to(&service.packet.data, &ctx.lock().unwrap().dest).unwrap();

                let mut buffer = [0; 1024];
                match socket.recv_from(&mut buffer) {
                    Ok((size, src)) => {
                        let recieved = u128::from_le_bytes(buffer[..size].try_into().unwrap());
                        services.insert(recieved, (service.clone(), true));
                    }
                    _ => {}
                }
            }
        }
    }

    fn send_rf(&self, ctx: &Context) {
        // todo()
    }


    pub fn run(&mut self) {
        let mut packet_count: u128 = 0;
        loop {
            for node in self.nodes.iter_mut() {
                if node.get_ctx().lock().unwrap().reached_time() {
                    let packets = node.gen_packets();
                    let mut services = HashMap::<u128, (Arc<Service>, bool)>::new();

                    for packet in packets {
                        services.insert(packet_count, (Arc::new(Service {
                            packet,
                            packet_id: packet_count,
                        }), false));

                        packet_count+=1;
                    }
                    let ctx = node.get_ctx().clone();
                    thread::spawn(|| { Self::send_udp(services, ctx); });
                }
            }

            packet_count = 0;
        }
    }

}