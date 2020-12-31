mod data;
mod layers;
mod pixels;

fn main() {
    println!("Hello, world!");
    //dbg!(pixels::Mapping::load_from_file("files/mappings/v6.json"));
    //data::init();

    let mut layer_manager = layers::Manager::new();
    layer_manager.sm();
}
