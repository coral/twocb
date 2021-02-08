//#[derive(Debug)]
pub trait Pattern {
    fn name(&self) -> String;

    fn init(&self);
}
