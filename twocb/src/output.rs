pub mod opc;
pub use self::opc::OPCOutput;

use vecmath;

pub trait Adapter {
    fn write(&mut self, data: &[vecmath::Vector4<f64>]);
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

    pub fn write(&mut self, data: &[vecmath::Vector4<f64>]) {
        for output in &mut self.outputs {
            output.write(data);
        }
    }
}
