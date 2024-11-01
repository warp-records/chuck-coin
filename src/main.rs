use std::fs;

fn main() {
    println!("Chuck coin: where a kid can be a kid!");
    println!("Take a coin kiddo:\n");
    println!("{}", fs::read_to_string("asciiart.txt").unwrap());
}
