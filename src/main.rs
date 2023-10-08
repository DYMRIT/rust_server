use std::env;

fn main() {

    println!("HTTP/1.1 200 OK\r");
    println!("Content-Type: text/plain\r\n");
    println!("Hello world!");
    let args: Vec<String> = env::args().collect();
    let query_string = args.get(1).unwrap_or(&String::new());
    println!("{:?}", query_string);
}