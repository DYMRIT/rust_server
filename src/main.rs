use std::env;

fn main() {

    println!("HTTP/1.1 200 OK\r");
    println!("Content-Type: text/plain\r\n");
    println!("Hello world!");
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
}