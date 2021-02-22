use vecmath;

mod add;

pub enum BlendModes {
    Add,
    Screen,
}

pub fn blend(
    mode: BlendModes,
    op1: Vec<vecmath::Vector3<f64>>,
    op2: Vec<vecmath::Vector3<f64>>,
    value: f64,
) -> Vec<vecmath::Vector3<f64>> {
    match mode {
        BlendModes::Add => {
            return add::add(op1, op2);
        }
        BlendModes::Screen => {
            return screen::screen(op1, op2);
        }
    }
}
