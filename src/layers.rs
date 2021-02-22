use twocb::engines::Pattern;

pub mod blending;

pub struct Manager {}

impl Manager {
    pub fn new() -> Manager {
        Manager {}
    }

    pub fn sm(&mut self) {}

    pub fn render(&mut self) {}
}

struct Link {
    step: Box<dyn Pattern>,
    parmeters: Vec<String>,
}

// pub trait Step {
//     fn init(&self);
//     fn query_parameters(&self) -> Vec<String>;
//     fn query_requirements(&self);
//     fn render(&self);
// }
