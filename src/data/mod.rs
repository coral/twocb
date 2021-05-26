use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

pub struct DataLayer {
    db: PickleDb,
}

impl DataLayer {
    pub fn new(dbpath: &str) -> Result<DataLayer, &'static str> {
        let db = PickleDb::load(
            dbpath,
            PickleDbDumpPolicy::DumpUponRequest,
            SerializationMethod::Json,
        );

        match db {
            Ok(db) => return Ok(DataLayer { db }),
            Err(_err) => {
                let newdb = PickleDb::new(
                    dbpath,
                    PickleDbDumpPolicy::AutoDump,
                    SerializationMethod::Json,
                );
                return Ok(DataLayer { db: newdb });
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
}
