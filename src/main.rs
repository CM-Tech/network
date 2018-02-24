use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

enum Action {
    Add(SocketAddr, TcpStream),
    Remove(SocketAddr),
    Broadcast(SocketAddr, String),
}

struct Server {
    connections: HashMap<SocketAddr, TcpStream>,
}

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
    let (tx, rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();

    loop {
        if let Ok((stream, addr)) = listener.accept() {
            thread::spawn(|| {
                handle_client(stream);
            });
        }
    }
}
