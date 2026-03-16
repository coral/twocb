use crate::output;
use ddp_rs::connection::DDPConnection;
use ddp_rs::protocol::{PixelConfig, ID};
use log::{info, warn};
use std::net::UdpSocket;

pub struct DDPOutput {
    connection: DDPConnection,
    buffer: Vec<u8>,
}

impl output::Adapter for DDPOutput {
    fn write(&mut self, data: &[vecmath::Vector4<f64>]) {
        self.buffer.clear();
        for pixel in data {
            self.buffer
                .push((pixel[0].clamp(0.0, 1.0) * 255.0) as u8);
            self.buffer
                .push((pixel[1].clamp(0.0, 1.0) * 255.0) as u8);
            self.buffer
                .push((pixel[2].clamp(0.0, 1.0) * 255.0) as u8);
        }
        if let Err(e) = self.connection.write(&self.buffer) {
            warn!("DDP send failed: {}", e);
        }
    }
}

impl DDPOutput {
    pub fn new(addr: &str, pixel_count: usize) -> anyhow::Result<DDPOutput> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let connection =
            DDPConnection::try_new(addr, PixelConfig::default(), ID::Default, socket)?;
        info!("DDP output to {}", addr);
        Ok(DDPOutput {
            connection,
            buffer: Vec::with_capacity(pixel_count * 3),
        })
    }
}
