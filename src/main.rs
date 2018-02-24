use std::net::{TcpListener, TcpStream};
use std::io::Write;

fn handle_client(mut stream: TcpStream) {
    stream.write(b"testing").unwrap();
    stream.flush().unwrap();
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("try connecting via `telnet localhost 8080`");
    // accept connections and process them serially
    loop {
        if let Ok((stream, addr)) = listener.accept() {
            handle_client(stream);
        }
    }
}
