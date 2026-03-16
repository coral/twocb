use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../files/types/")]
pub enum ParamKind {
    Slider,
    HsvPicker,
    RgbPicker,
    Toggle,
    Number,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../files/types/")]
pub enum ParamValue {
    Float(f64),
    Color3(f64, f64, f64),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../files/types/")]
pub struct PatternParam {
    pub name: String,
    pub kind: ParamKind,
    pub value: ParamValue,
}

/// Prefix conventions for parameter discovery (Pixelblaze-compatible).
pub const PARAM_PREFIXES: &[(&str, ParamKind)] = &[
    ("slider", ParamKind::Slider),
    ("hsvPicker", ParamKind::HsvPicker),
    ("rgbPicker", ParamKind::RgbPicker),
    ("toggle", ParamKind::Toggle),
    ("number", ParamKind::Number),
];

impl ParamKind {
    pub fn default_value(&self) -> ParamValue {
        match self {
            ParamKind::Slider => ParamValue::Float(0.5),
            ParamKind::HsvPicker => ParamValue::Color3(0.0, 1.0, 1.0),
            ParamKind::RgbPicker => ParamValue::Color3(1.0, 1.0, 1.0),
            ParamKind::Toggle => ParamValue::Bool(false),
            ParamKind::Number => ParamValue::Float(0.0),
        }
    }
}

// Allow cloning ParamKind from the const slice
impl Copy for ParamKind {}
