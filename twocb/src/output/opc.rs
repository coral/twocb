use crate::output;
use log::{info, warn};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use zeroconf::prelude::*;
use zeroconf::{MdnsService, TxtRecord};

const SET_PIXEL_COLORS: u8 = 0x00;
const BROADCAST_CHANNEL: u8 = 0x00;

pub struct OPCOutput {
    addr: SocketAddr,
    send_fails: usize,
    reconnection_attempts: usize,
    stream: Option<tokio::net::TcpStream>,
    buffer: Vec<u8>,
}

impl output::Adapter for OPCOutput {
    fn write(&mut self, data: &[vecmath::Vector4<f64>]) {
        self.buffer.clear();
        self.buffer.push(BROADCAST_CHANNEL);
        self.buffer.push(SET_PIXEL_COLORS);
        (data.len() as u16 * 3).to_be_bytes().iter().for_each(|x| {
            self.buffer.push(*x);
        });
        data.iter().for_each(|pixel| {
            self.buffer.push((pixel[0].clamp(0.0, 1.0) * 255.) as u8);
            self.buffer.push((pixel[1].clamp(0.0, 1.0) * 255.) as u8);
            self.buffer.push((pixel[2].clamp(0.0, 1.0) * 255.) as u8);
        });

        if let Some(ref s) = self.stream {
            {
                match s.try_write(self.buffer.as_slice()) {
                    Err(e) => {
                        warn!("Could not send OPC message with error: {}.", e);
                        self.send_fails = self.send_fails + 1;
                    }
                    _ => (),
                }
            }
        }
    }
}

impl OPCOutput {
    pub fn new(addr: SocketAddr) -> OPCOutput {
        return OPCOutput {
            addr,
            send_fails: 0,
            reconnection_attempts: 0,
            stream: None,
            buffer: vec![0; (2000 * 3) + 4],
        };
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let stream = TcpStream::connect(self.addr).await?;
        self.stream = Some(stream);
        info!("Connected to OPC Server: {}", self.addr);
        Ok(())
    }
}

pub struct OPCDiscovery {}

impl OPCDiscovery {
    fn advertise() {
        let _service = MdnsService::new("_opc._tcp", 3030);
        let _txt_record = TxtRecord::new();
    }
}
