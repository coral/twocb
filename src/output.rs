pub mod opc;
pub use self::opc::OPCOutput;
use std::rc::Rc;

use vecmath;

pub trait Adapter {
    fn write(&mut self, data: Vec<vecmath::Vector4<f64>>);
}

#[derive(Clone)]
pub struct OutputManager {
    outputs: Vec<Rc<dyn Adapter>>,
}

impl OutputManager {
    pub fn new() -> OutputManager {
        OutputManager {
            outputs: Vec::new(),
        }
    }

    pub fn add(&mut self, output: Rc<dyn Adapter>) {
        self.outputs.push(output);
    }

    pub fn write(self, data: Vec<vecmath::Vector4<f64>>) {
        for output in self.outputs {
            output.borrow().write(data);
        }
    }
}
