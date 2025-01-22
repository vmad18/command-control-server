use crate::lib::util::{Context, Node, Packet};
use std::collections::HashMap;
use std::hash::Hash;
use std::net::UdpSocket;
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Service {
    packet: Packet,
    packet_id: u16,
}

pub struct CommandControl {
    nodes: Vec<Box<dyn Node>>,
}

pub enum Channel {
    UDP,
    TCP,
    RF,
}

impl CommandControl {
    pub fn create() -> Self {
        CommandControl { nodes: vec![] }
    }

    pub fn send(&self) {}

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

    pub fn curr_network() -> String {
        let result = Command::new("nmcli")
            .args(&["-g", "GENERAL.CONNECTION", "device", "show", "wlan0"])
            .output()
            .expect("Could not get network");

        let stdout = String::from_utf8_lossy(&result.stdout);

        println!("{}", stdout.to_string().trim());

        stdout.to_string().trim().to_string()
    }

    fn connect_first_time(ssid: &String, pw: &String) {
        let result = Command::new("nmcli")
            .args(&["d", "wifi", "connect", ssid, "password", pw])
            .output()
            .expect("Could not connect to access point");
    }

    fn switch_network(ssid: &String) {
        let result = Command::new("nmcli")
            .args(&["connection", "up", ssid])
            .output()
            .expect("Could not switch network");
    }

    fn send_udp(mut services: HashMap<u16, (Arc<Service>, bool)>, ctx: Arc<Mutex<Context>>) {
        Self::switch_network(&ctx.lock().unwrap().dest.1);

        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        while !Self::visited::<u16>(&services) {
            for id in services.clone().keys() {
                let (service, visited) = services.get(id).unwrap();
                if *visited {
                    continue;
                }

                socket
                    .send_to(
                        &service.packet.data[..service.packet.size + 2],
                        &ctx.lock().unwrap().dest.0,
                    )
                    .unwrap();

                let mut buffer = [0; 4096];
                match socket.recv_from(&mut buffer) {
                    Ok((size, _src)) => {
                        let recieved = u16::from_le_bytes(buffer[..size].try_into().unwrap());
                        services.insert(recieved, (service.clone(), true));
                    }
                    _ => {}
                }
            }
        }
    }

    fn send_rf(ctx: &Context) {
        // todo()
    }

    pub fn run(&mut self) {
        let mut packet_count: u16 = 0;
        loop {
            for node in self.nodes.iter_mut() {
                if node.get_ctx().lock().unwrap().reached_time() {
                    let packets = node.gen_packets();
                    let mut services = HashMap::<u16, (Arc<Service>, bool)>::new();

                    for mut packet in packets {
                        let b1 = (packet_count >> 8) as u8;
                        let b2 = (packet_count & 0xff) as u8;

                        println!("{} {} {}", b1, b2, packet_count);

                        packet.data[packet.size] = b1;
                        packet.data[packet.size + 1] = b2;

                        println!("{:?}", packet.data);

                        services.insert(
                            packet_count,
                            (
                                Arc::new(Service {
                                    packet,
                                    packet_id: packet_count,
                                }),
                                false,
                            ),
                        );

                        packet_count += 1;
                    }

                    let ctx = node.get_ctx().clone();

                    // match ctx.clone().lock().unwrap().send_over {
                    //     Channel::UDP => {
                    //         thread::spawn(|| {
                    //             Self::send_udp(services, ctx);
                    //         });
                    //     }
                    //     _ => {}
                    // }
                }
            }

            packet_count = 25010;
        }
    }
}
