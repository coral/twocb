use vecmath;

//#[derive(Debug)]
pub trait Pattern {
    fn name(&self) -> String;

    fn process(&self) -> Vec<vecmath::Vector4<f64>>;
}
