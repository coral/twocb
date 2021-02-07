use crate::engines::Pattern;
pub struct Strobe {}

impl Pattern for Strobe {
    fn name() -> &str {
        return "strobe";
    }
}
