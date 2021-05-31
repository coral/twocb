use anyhow::Result;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct DataLayer {
    db: PickleDb,
    subscribed_keys: HashMap<String, mpsc::Sender<Vec<u8>>>,
}

impl DataLayer {
    pub fn new(dbpath: &str) -> Result<DataLayer, &'static str> {
        let db = PickleDb::load(
            dbpath,
            PickleDbDumpPolicy::PeriodicDump(Duration::from_secs(30)),
            SerializationMethod::Json,
        );

        match db {
            Ok(db) => {
                return Ok(DataLayer {
                    db,
                    subscribed_keys: HashMap::new(),
                })
            }
            Err(_err) => {
                let newdb = PickleDb::new(
                    dbpath,
                    PickleDbDumpPolicy::AutoDump,
                    SerializationMethod::Json,
                );
                return Ok(DataLayer {
                    db: newdb,
                    subscribed_keys: HashMap::new(),
                });
            }
        }
    }

    pub async fn subscribe(&mut self, key: &str) -> Result<mpsc::Receiver<Vec<u8>>, &'static str> {
        let (tx, mut rx) = mpsc::channel(10);
        match self.subscribed_keys.insert(key.to_string(), tx) {
            None => Ok(rx),
            Some(v) => Err("key already exists"),
        }
    }

    pub async fn seed_state() {}

    pub fn get_state(&self, key: &str) -> Option<Vec<u8>> {
        self.db.get::<Vec<u8>>(key)
    }

    pub fn write_state(&mut self, key: &str, value: &[u8]) {
        self.db.set(key, &value);
    }
}
