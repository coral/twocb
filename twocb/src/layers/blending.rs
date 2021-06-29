use serde::{Deserialize, Serialize};
use strum_macros;
use vecmath;

mod add;
mod screen;
mod subtract;

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, strum_macros::ToString, strum_macros::EnumString,
)]
#[allow(dead_code)]
pub enum BlendModes {
    Add,
    Subtract,
    Screen,
}

pub fn blend(
    mode: BlendModes,
    op1: Vec<vecmath::Vector4<f64>>,
    op2: Vec<vecmath::Vector4<f64>>,
    _value: f64,
) -> Vec<vecmath::Vector4<f64>> {
    match mode {
        BlendModes::Add => {
            return add::add(op1, op2);
        }
        BlendModes::Subtract => {
            return subtract::sub(op1, op2);
        }
        BlendModes::Screen => {
            return screen::screen(op1, op2);
        }
    }
}

pub fn scale(mut input: Vec<vecmath::Vector4<f64>>, val: f64) -> Vec<vecmath::Vector4<f64>> {
    for v in input.iter_mut() {
        *v = vecmath::vec4_mul(*v, [val, val, val, 1.0]);
    }
    return input;
}
