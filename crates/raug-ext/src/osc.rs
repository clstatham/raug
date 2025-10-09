use std::{
    collections::HashMap,
    net::{ToSocketAddrs, UdpSocket},
};

use raug::prelude::Param;

pub use rosc::OscType;

pub type OscRule = dyn FnMut(&[rosc::OscType], ParamProxy) + Send + Sync;

#[derive(Clone, Copy)]
pub struct ParamProxy<'a> {
    params: &'a HashMap<String, Param<f32>>,
}

impl<'a> ParamProxy<'a> {
    pub fn new(params: &'a HashMap<String, Param<f32>>) -> Self {
        Self { params }
    }

    pub fn get(&self, name: &str) -> Option<&Param<f32>> {
        self.params.get(name)
    }
}

pub struct OscClient {
    pub socket: UdpSocket,
    pub params: HashMap<String, Param<f32>>,
    pub rules: HashMap<String, Vec<Box<OscRule>>>,
}

impl OscClient {
    pub fn bind(addr: impl ToSocketAddrs) -> Self {
        let socket = UdpSocket::bind(addr).expect("could not bind to address");
        socket
            .set_nonblocking(true)
            .expect("could not set non-blocking");
        Self {
            socket,
            params: HashMap::new(),
            rules: HashMap::new(),
        }
    }

    pub fn register_param(&mut self, name: &str, initial_value: f32) -> Param<f32> {
        let param = Param::new(initial_value);
        self.params.insert(name.to_string(), param.clone());
        self.rules.insert(name.to_string(), vec![]);
        param
    }

    pub fn add_rule<F>(&mut self, address: &str, rule: F)
    where
        F: FnMut(&[rosc::OscType], ParamProxy) + Send + Sync + 'static,
    {
        self.rules
            .entry(address.to_string())
            .or_default()
            .push(Box::new(rule));
    }

    pub fn poll(&mut self) {
        let mut buf = [0u8; 1024];
        while let Ok((size, _src)) = self.socket.recv_from(&mut buf) {
            if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                self.handle_packet(packet);
            }
        }
    }

    pub fn listen(&mut self) {
        loop {
            self.poll();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    pub fn spawn(mut self) -> std::thread::JoinHandle<Self> {
        std::thread::spawn(move || {
            self.listen();
            self
        })
    }

    fn handle_packet(&mut self, packet: rosc::OscPacket) {
        match packet {
            rosc::OscPacket::Message(msg) => self.handle_message(msg),
            rosc::OscPacket::Bundle(bundle) => {
                for p in bundle.content {
                    self.handle_packet(p);
                }
            }
        }
    }

    fn handle_message(&mut self, msg: rosc::OscMessage) {
        if let Some(rules) = self.rules.get_mut(&msg.addr) {
            let param = ParamProxy::new(&self.params);
            for rule in rules {
                rule(&msg.args, param);
            }
        }
    }
}
