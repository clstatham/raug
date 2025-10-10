use std::{
    collections::HashMap,
    net::{ToSocketAddrs, UdpSocket},
    thread::JoinHandle,
    time::Duration,
};

use raug::prelude::Param;

pub use rosc::OscType;
use rosc::{OscMessage, OscPacket};

pub type OscRule = dyn FnMut(&[rosc::OscType], ParamProxy) + Send + Sync;

#[derive(Clone, Copy)]
pub struct ParamProxy<'a> {
    params: &'a HashMap<String, Param<f32>>,
}

impl<'a> ParamProxy<'a> {
    pub(crate) fn new(params: &'a HashMap<String, Param<f32>>) -> Self {
        Self { params }
    }

    pub fn get(&self, name: &str) -> Option<&Param<f32>> {
        self.params.get(name)
    }
}

pub struct OscClient {
    socket: UdpSocket,
    params: HashMap<String, Param<f32>>,
    rules: HashMap<String, Vec<Box<OscRule>>>,
}

impl OscClient {
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            params: HashMap::new(),
            rules: HashMap::new(),
        })
    }

    pub fn register_param(&mut self, name: &str, initial_value: f32) -> Param<f32> {
        let param = Param::new(initial_value);
        self.params.insert(name.to_string(), param.clone());
        self.rules.insert(name.to_string(), vec![]);
        param
    }

    pub fn add_rule<F>(&mut self, address: &str, rule: F)
    where
        F: FnMut(&[OscType], ParamProxy) + Send + Sync + 'static,
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

    pub fn listen_forever(&mut self) {
        loop {
            self.poll();
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn spawn(mut self) -> JoinHandle<Self> {
        std::thread::spawn(move || {
            self.listen_forever();
            self
        })
    }

    fn handle_packet(&mut self, packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => self.handle_message(msg),
            OscPacket::Bundle(bundle) => {
                for p in bundle.content {
                    self.handle_packet(p);
                }
            }
        }
    }

    fn handle_message(&mut self, msg: OscMessage) {
        if let Some(rules) = self.rules.get_mut(&msg.addr) {
            let param = ParamProxy::new(&self.params);
            for rule in rules {
                rule(&msg.args, param);
            }
        }
    }
}
