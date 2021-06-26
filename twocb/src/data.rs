use anyhow::Result;
use sled;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct DataLayer {
    pub db: sled::Db,

    pub links: sled::Tree,
    pub state: sled::Tree,
    subscribed_keys: HashMap<String, mpsc::Sender<Vec<u8>>>,
}

impl DataLayer {
    pub fn new(dbpath: &str) -> Result<DataLayer, &'static str> {
        match sled::open(dbpath) {
            Ok(db) => {
                let state = db.open_tree("state").unwrap();
                let links = db.open_tree("layers").unwrap();
                return Ok(DataLayer {
                    db,
                    state,
                    links,
                    subscribed_keys: HashMap::new(),
                });
            }
            Err(err) => {
                dbg!(err);
                return Err("FML");
            }
        }
    }

    pub async fn subscribe(&mut self, key: &str) -> Result<mpsc::Receiver<Vec<u8>>, &'static str> {
        let (tx, mut rx) = mpsc::channel(10);
        match self.subscribed_keys.insert(key.to_string(), tx) {
            None => Ok(rx),
            Some(_) => Err("key already exists"),
        }
    }

    pub async fn seed_state() {}

    pub fn get_state(&self, key: &str) -> Option<Vec<u8>> {
        match self.state.get(key) {
            Ok(v) => match v {
                Some(d) => {
                    return Some(d.to_vec());
                }
                None => return None,
            },
            Err(_e) => return None,
        }
    }

    pub fn get_states(&self) -> HashMap<String, String> {
        let mut packaged_states = HashMap::new();

        for state in self.state.iter() {
            match state {
                Ok((k, v)) => {
                    let key = std::str::from_utf8(&k).unwrap().to_string();
                    let val = std::str::from_utf8(&v).unwrap().to_string();

                    packaged_states.insert(key, val);
                }
                _ => {}
            }
        }

        packaged_states
    }

    pub fn write_state(&mut self, key: &str, value: &[u8]) {
        self.state.insert(key, value);
    }

    //pub fn get_layers() -> Vec<u8> {}

    pub fn get_layers(&self) -> HashMap<String, String> {
        let mut packaged_states = HashMap::new();

        for state in self.links.iter() {
            match state {
                Ok((k, v)) => {
                    let key = std::str::from_utf8(&k).unwrap().to_string();
                    let val = std::str::from_utf8(&v).unwrap().to_string();

                    packaged_states.insert(key, val);
                }
                _ => {}
            }
        }

        packaged_states
    }

    pub fn write_layer(&mut self, key: &str, value: &[u8]) {
        self.links.insert(key, value);
    }
}
