use std::process::Command;

fn main() {

    println!("HTTP/1.1 200 OK\r");
    println!("Content-Type: text/plain\r\n");
    println!("Hello world!");

    let result = Command::new("curl")
        .arg("https://jsonplaceholder.typicode.com/posts/2")
        .output()
        .expect("failed to execute command");
    println!("status: {}", result.status);
    println!("stdout: {}", String::from_utf8_lossy(&result.stdout));
}