mod data;
mod pixels;

fn main() {
    println!("Hello, world!");
    dbg!(pixels::Mapping::load_from_file("files/mappings/v6.json"));
    data::init();
}
