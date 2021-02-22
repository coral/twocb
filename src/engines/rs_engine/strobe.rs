use crate::engines::pattern;
pub struct Strobe {}

impl pattern::Pattern for Strobe {
    fn name(&self) -> String {
        return "strobe".to_string();
    }

    fn frame(&self) {}

    fn process(&self) {}
}
