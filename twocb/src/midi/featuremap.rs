use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
pub type Routes = Vec<Route>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    #[serde(rename = "in")]
    pub in_field: String,
    pub out: Out,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Out {
    #[serde(rename = "type")]
    pub type_field: OutType,
    pub path: String,

    #[serde(default = "min")]
    pub min: f64,
    #[serde(default = "max")]
    pub max: f64,

    #[serde(skip)]
    pub pattern_key: String,
    #[serde(skip)]
    pub state_key: String,
}

fn min() -> f64 {
    0.0
}
fn max() -> f64 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutType {
    Opacity,
    Parameter,
}
impl Default for OutType {
    fn default() -> Self {
        OutType::Opacity
    }
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Route>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut map: Vec<Route> = serde_json::from_reader(reader)?;

    for route in &mut map {
        let p = Path::new(&route.out.path);

        route.out.pattern_key = p.parent().unwrap().to_string_lossy().to_string();
        route.out.state_key = p.file_name().unwrap().to_string_lossy().to_string()
    }
    Ok(map)
}
