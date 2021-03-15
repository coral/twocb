pub mod opc;
pub use self::opc::OPCOutput;

use vecmath;

pub trait Adapter {
    fn write(&mut self, data: Vec<vecmath::Vector4<f64>>);
}
