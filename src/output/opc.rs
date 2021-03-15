use crate::output;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const SET_PIXEL_COLORS: u8 = 0x00;
const BROADCAST_CHANNEL: u8 = 0x00;

pub struct OPCOutput {
    addr: SocketAddr,
    stream: Option<Arc<Mutex<tokio::net::TcpStream>>>,
    buffer: Vec<u8>,
}

impl output::Adapter for OPCOutput {
    fn write(&mut self, data: Vec<vecmath::Vector4<f64>>) {
        self.buffer.clear();
        self.buffer.push(BROADCAST_CHANNEL);
        self.buffer.push(SET_PIXEL_COLORS);
        (data.len() as u16).to_be_bytes().iter().for_each(|x| {
            self.buffer.push(*x);
        });
        data.iter().for_each(|pixel| {
            self.buffer.push((pixel[0].clamp(0.0, 1.0) * 255.) as u8);
            self.buffer.push((pixel[1].clamp(0.0, 1.0) * 255.) as u8);
            self.buffer.push((pixel[2].clamp(0.0, 1.0) * 255.) as u8);
        });

        if let Some(ref s) = self.stream {
            {
                s.clone().lock().unwrap().try_write(self.buffer.as_slice());
            }
        }
    }
}

impl OPCOutput {
    pub fn new(addr: SocketAddr) -> OPCOutput {
        return OPCOutput {
            addr,
            stream: None,
            buffer: vec![0; (2000 * 3) + 4],
        };
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let mut stream = TcpStream::connect(self.addr).await?;
        self.stream = Some(Arc::new(Mutex::new(stream)));
        Ok(())
    }
}
