use twocb::engines;
use twocb::engines::Engine;

fn main() {
    let mut rse = engines::RSEngine::new();
    rse.bootstrap().unwrap();

    rse.hello();
    let patterns = rse.list();

    for i in patterns.iter() {
        println!("{:?}", i.name());
    }
}
