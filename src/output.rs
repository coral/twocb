pub mod opc;
pub use self::opc::OPCOutput;

use vecmath;

pub trait Adapter {
    fn write(&mut self, data: Vec<vecmath::Vector4<f64>>);
}

pub struct OutputManager {
    outputs: Vec<Box<dyn Adapter>>,
}

impl OutputManager {
    pub fn new() -> OutputManager {
        OutputManager {
            outputs: Vec::new(),
        }
    }

    pub fn add(&mut self, output: Box<dyn Adapter>) {
        self.outputs.push(output);
    }

    pub fn write(self, data: Vec<vecmath::Vector4<f64>>) {}
}
