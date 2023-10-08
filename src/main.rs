use std::env;

fn main() {

    println!("HTTP/1.1 200 OK\r");
    println!("Content-Type: text/plain\r\n");
    println!("Hello world!");
    let l = env::args_os();
    for argument in l {
        println!("{argument:?}");
    }
}