//#[derive(Debug)]
pub trait Pattern {
    fn name(&self) -> String;

    fn process(&self);
}
