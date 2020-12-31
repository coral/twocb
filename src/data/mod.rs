use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

pub fn init() {
    // let mut db = PickleDb::new(
    //     "files/example.db",
    //     PickleDbDumpPolicy::AutoDump,
    //     SerializationMethod::Json,
    // );
    let db = PickleDb::load(
        "files/example.db",
        PickleDbDumpPolicy::DumpUponRequest,
        SerializationMethod::Json,
    )
    .unwrap();

    //db.set("key1", &100).unwrap();

    println!("The value of key1 is: {}", db.get::<i32>("key1").unwrap());
}
