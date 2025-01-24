use crate::lib::util::{Context, Node, Packet};
use std::collections::HashMap;
use std::hash::Hash;
use std::net::UdpSocket;
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use super::util::PACKET_SIZE;

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

    fn validate_resp<T: Eq + Hash>(map: &HashMap<T, (Arc<Service>, bool)>) -> bool {
        for key in map.keys() {
            if !(&map.get(key).unwrap().1) {
                return false;
            }
        }

        true
    }

    fn curr_network() -> String {
        let result = Command::new("nmcli")
            .args(&["-g", "GENERAL.CONNECTION", "device", "show", "wlp2s0"])
            .output()
            .expect("Could not get network");

        let stdout = String::from_utf8_lossy(&result.stdout);

        stdout.to_string().trim().to_string()
    }

    pub fn connect_ap_first_time(ssid: &String, pw: &String, ap_ip: &String, ip: &String) {
        let curr_network = Self::curr_network();

        // Command::new("nmcli")
        //     .args(&["d", "wifi", "connect", ssid, "password", pw])
        //     .output()
        //     .expect("Could not connect to access point");
         
        Command::new("nmcli").args(&["connection", "modify", ssid, "ipv4.addresses", ip, "ipv4.gateway", ap_ip, "ipv4.method", "manual"]);
        Self::switch_network(ssid);
        Self::switch_network(&curr_network);
    }

    fn switch_network(ssid: &String) {
        println!("switching to network {}", ssid);
        let result = Command::new("nmcli")
            .args(&["connection", "up", ssid])
            .output()
            .expect("Could not switch network").stdout;
    }

    fn send_udp(mut services: HashMap<u16, (Arc<Service>, bool)>, ctx: Arc<Mutex<Context>>) {
        let curr_network = Self::curr_network().to_string();
        
        Self::switch_network(&ctx.lock().unwrap().dest.2);
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let mut attempts = 10;
        while !Self::validate_resp::<u16>(&services) && attempts > 0 {
            attempts -= 1;
            for id in services.clone().keys() {
                let (service, visited) = services.get(id).unwrap();
                
                if *visited { continue; }
                
                let shared_ctx = &ctx.lock().unwrap();

                let ip_addr = shared_ctx.dest.0.clone();
                let port = shared_ctx.dest.1.clone();
                let ip_port = format!("{}:{}", ip_addr, port);

                println!("sending to -> {}", ip_port);
                match socket
                    .send_to(
                        &service.packet.data[..service.packet.size + 2],
                        ip_port,
                    ) {
                        Ok(_) => {},
                        Err(_) => {
                            println!("could not relay data!");
                            continue;
                        }
                    }
                    
                let mut buffer = [0; PACKET_SIZE];
                match socket.recv_from(&mut buffer) {
                    Ok((_size, _src)) => {
                        let rcv_id: u16 = ((buffer[0] as u16) << 8) | ((buffer[1] as u16) & 0x00ff);
                        println!("verified packet: {}", rcv_id);
                        services.insert(rcv_id, (service.clone(), true));
                    }
                    _ => {}
                }
            }
        }

        Self::switch_network(&curr_network);
        ctx.clone().lock().unwrap().finished = true;
    }

    fn send_rf(ctx: &Context) {
        // todo()
    }

    pub fn run(&mut self) {
        println!("Running Command 'n Control Server...");
        let mut packet_count: u16 = 1;
        loop {
            for node in self.nodes.iter_mut() {
                if node.get_ctx().lock().unwrap().reached_time() {
                    if node.get_ctx().lock().unwrap().finished {
                        let packets = node.gen_packets();
                        let mut services = HashMap::<u16, (Arc<Service>, bool)>::new();

                        for mut packet in packets {
                            let b1 = (packet_count >> 8) as u8;
                            let b2 = (packet_count & 0xff) as u8;

                            packet.data[packet.size] = b1;
                            packet.data[packet.size + 1] = b2;

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
                        ctx.clone().lock().unwrap().finished = false;
                        match ctx.clone().lock().unwrap().send_over {
                            Channel::UDP => {
                                thread::spawn(|| {
                                    Self::send_udp(services, ctx);
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }

            packet_count = 1;
        }
    }
}