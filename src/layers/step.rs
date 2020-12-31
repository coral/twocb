pub trait Step {
    fn Init(&self);
    fn RegisterParameters(&self) -> &[str];
}
