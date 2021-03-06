use twocb::engines;
use twocb::engines::Engine;
use twocb::layers::Manager;

fn main() {
    let mut rse = engines::RSEngine::new();
    rse.bootstrap().unwrap();

    rse.hello();
    let patterns = rse.list();

    for i in patterns.iter() {
        dbg!(i.name());
        //println!("{:?}", i.name());
    }
}
