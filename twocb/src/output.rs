pub mod opc;
pub mod ddp;

pub use self::opc::OPCOutput;
pub use self::ddp::DDPOutput;

use vecmath;

pub trait Adapter {
    fn write(&mut self, data: &[vecmath::Vector4<f64>]);
}

struct OutputEntry {
    adapter: Box<dyn Adapter>,
    start: usize,
    end: usize,
}

pub struct OutputManager {
    outputs: Vec<OutputEntry>,
}

impl OutputManager {
    pub fn new() -> OutputManager {
        OutputManager {
            outputs: Vec::new(),
        }
    }

    pub fn add(&mut self, adapter: Box<dyn Adapter>, start: usize, end: usize) {
        self.outputs.push(OutputEntry {
            adapter,
            start,
            end,
        });
    }

    pub fn write(&mut self, data: &[vecmath::Vector4<f64>]) {
        for entry in &mut self.outputs {
            let start = entry.start.min(data.len());
            let end = entry.end.min(data.len());
            if start < end {
                entry.adapter.write(&data[start..end]);
            }
        }
    }
}
