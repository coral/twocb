use anyhow::Result;

#[derive(Debug)]
pub struct Pattern {
    pub name: String,
}

pub trait Engine {
    fn bootstrap(&mut self) -> anyhow::Result<()>;
    fn list(&mut self) -> Vec<Pattern>;
    //fn load(&mut self, p: Pattern) -> anyhow::Result<()>;
    // fn process(&mut self) -> Vec<f64>;
}
