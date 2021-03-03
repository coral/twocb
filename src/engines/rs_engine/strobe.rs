use crate::engines::pattern;
pub struct Strobe {}

impl pattern::Pattern for Strobe {
    fn name(&self) -> String {
        return "strobe".to_string();
    }

    fn process(&self) -> Vec<vecmath::Vector4<f64>> {
        return vec![[1.0; 4]; 100];
    }
}
