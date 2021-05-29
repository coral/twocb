use std::fs;

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub endpoints: Endpoints,
    pub audio: Audio,
    pub database: String,
    pub api: Api,
    pub mapping: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    pub opc: Vec<Opc>,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opc {
    pub host: String,
    pub port: i64,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: u16,

    pub tempo: Tempo,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tempo {
    pub confidence_limit: f32,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Api {
    pub host: String,
    pub port: u16,
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let data = fs::read_to_string(path)?;
    let cfg: Config = serde_json::from_str(&data)?;
    Ok(cfg)
}
