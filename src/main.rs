pub mod block;
pub mod tx;
use std::fs;

fn main() {
    println!("Project \"Prismo Peso\"\n");
    println!("{}\n", fs::read_to_string("asciiart.txt").unwrap());
    println!("\"Hey Jake, see that? Monkey's paw..\"");

}
