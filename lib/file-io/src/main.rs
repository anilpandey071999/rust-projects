use std::env::args;

fn main() {
    let arg = args().collect::<Vec<String>>();
    println!("{:?}", arg);
    if arg.len() == 4 {
        println!("");
    }
    println!("Hello, world!");
}
