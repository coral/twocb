use serde::{Deserialize, Serialize};
use serde_json::*;
use std::env;
use std::fs;
use vecmath::Vector3;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pixel {
    #[serde(rename = "I")]
    pub index: i64,
    #[serde(rename = "A")]
    pub active: bool,
    #[serde(rename = "O")]
    pub coordinate: vecmath::Vector3<f64>,
    #[serde(rename = "N")]
    pub normal: vecmath::Vector3<f64>,
}

pub struct Mapping {
    pixels: Vec<Pixel>,
}

impl Mapping {
    pub fn load_from_file(filename: &str) -> Result<Vec<Pixel>> {
        let contents = fs::read_to_string(filename).expect("Could not read mapping");
        let v: Vec<Pixel> = serde_json::from_str(&contents)?;
        Ok(v)
    }
}
