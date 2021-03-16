use crate::engines::pattern;

pub struct Strobe {
    lit: bool,
}

impl pattern::Pattern for Strobe {
    fn name(&self) -> String {
        return "strobe".to_string();
    }

    fn process(&mut self) -> Vec<vecmath::Vector4<f64>> {
        let mut f = 0.0;
        if self.lit {
            f = 1.0;
            self.lit = false;
        } else {
            self.lit = true;
        }
        return vec![[f, f, f, 1.0]; 864];
    }
}

impl Strobe {
    pub fn new() -> Strobe {
        Strobe { lit: false }
    }

    pub fn name() -> String {
        return "strobe".to_string();
    }
}
