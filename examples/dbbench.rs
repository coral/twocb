use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::time::{Instant};

pub fn main() {
    // let mut db = PickleDb::new(
    //     "files/example.db",
    //     PickleDbDumpPolicy::AutoDump,
    //     SerializationMethod::Json,
    // );

    let mut db = PickleDb::new(
        "bench.db",
        PickleDbDumpPolicy::DumpUponRequest,
        SerializationMethod::Json,
    );

    let now = Instant::now();
    for x in 0..10000000 {
        db.set("key1", &x).unwrap();
    }
    println!("{}", now.elapsed().as_millis());
    println!("The value of key1 is: {}", db.get::<i32>("key1").unwrap());

    db.dump();
}
