use serde::{Deserialize, Serialize};
use serde_json::*;
use std::env;
use std::fs;
use vecmath;

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

#[allow(dead_code)]
impl Pixel {
    pub fn top(&self) -> bool {
        return self.coordinate[1] == 0.0;
    }

    pub fn bottom(&self) -> bool {
        return self.coordinate[1] == 1.0;
    }

    pub fn left(&self) -> bool {
        return self.coordinate[0] == 0.0;
    }

    pub fn right(&self) -> bool {
        return self.coordinate[0] == 1.0;
    }

    pub fn front(&self) -> bool {
        return self.coordinate[2] == 0.0;
    }

    pub fn back(&self) -> bool {
        return self.coordinate[2] == 1.0;
    }

    pub fn position_in_tube(&self) -> f64 {
        let mut c = self.coordinate;
        c[0] *= self.normal[0];
        c[1] *= self.normal[1];
        c[2] *= self.normal[2];

        return vecmath::vec3_len(c);
    }
}
