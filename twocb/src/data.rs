use anyhow::Result;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct DataLayer {
    db: PickleDb,
    subscribed_keys: HashMap<String, mpsc::Sender<Vec<u8>>>,
}

impl DataLayer {
    pub fn new(dbpath: &str) -> Result<DataLayer, &'static str> {
        let db = PickleDb::load(
            dbpath,
            PickleDbDumpPolicy::DumpUponRequest,
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

    pub fn woo(&mut self) {
        self.db.set("key1", &100).unwrap();
        println!(
            "The value of key1 is: {}",
            self.db.get::<i32>("key1").unwrap()
        );
    }

    pub async fn subscribe(&mut self, key: &str) -> Result<mpsc::Receiver<Vec<u8>>, &'static str> {
        //self.db.get::<Vec<u8>>(key).unwrap()
        let (tx, mut rx) = mpsc::channel(10);
        match self.subscribed_keys.insert(key.to_string(), tx) {
            None => Ok(rx),
            Some(v) => Err("key already exists"),
        }
        //self.seed_key(key).await;
    }

    //async fn seed_key(&self, key: &str) -> anyhow::Result<()> {
    // match self.db.get::<Vec<u8>>(key) {
    //     Some(data) => match self.subscribed_keys.get(key) {
    //         Some(sub) => match sub.send(data).await {
    //             Ok(()) => return Ok(())
    //             Err(v) => {}
    //         },
    //         None => return println!("failed to seed key: {}", key),
    //     },
    //     None => {
    //         println!("Could not find key in db: {}", key)
    //     }
    // }
    //}
}
