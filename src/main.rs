use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    stream.write(b"testing").unwrap();
    stream.flush().unwrap();
    loop {
        let mut buf = [0; 4096];
        if let Ok(n) = stream.read(&mut buf) {
            stream.write(&buf[0..n]).unwrap();
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("try connecting via `telnet localhost 8080`");
    loop {
        if let Ok((stream, addr)) = listener.accept() {
            thread::spawn(|| {
                handle_client(stream);
            });
        }
    }
}
