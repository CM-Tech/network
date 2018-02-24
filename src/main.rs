use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

pub struct Connection {
    pub addr: SocketAddr,
    pub stream: TcpStream,
}

enum Action {
    Add(SocketAddr, TcpStream),
    Remove(SocketAddr),
    Broadcast(SocketAddr, String),
}

struct Server {
    connections: HashMap<SocketAddr, TcpStream>,
}

impl Server {
    fn broadcast(&mut self, from: &SocketAddr, msg: &[u8]) {
        println!(
            "broadcasting msg: {}",
            String::from_utf8(msg.to_vec()).unwrap()
        );
        for (addr, mut connection) in self.connections.iter_mut() {
            if *from == *addr {
                continue;
            }
            connection.write(msg).ok();
            connection.flush().ok();
        }
    }

    fn add_connection(&mut self, addr: &SocketAddr, stream: TcpStream) {
        self.connections.insert(*addr, stream);
        let msg = format!(
            "({} connections) ----- new connection from {} -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
        self.broadcast(addr, (msg + "\n").as_bytes());
    }

    fn remove_connection(&mut self, addr: &SocketAddr) {
        self.connections.remove(addr);
        let msg = format!(
            "({} connections) ----- {} is disconnected -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
        self.broadcast(addr, (msg + "\n").as_bytes());
    }
}
fn handle_client(mut stream: TcpStream, addr: SocketAddr, sender: Sender<Action>) {
    stream.write(b"testing\n").unwrap();
    stream.flush().unwrap();
    'read: loop {
        let mut buf = [0; 4096];
        if let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break 'read;
            }
            sender
                .send(Action::Broadcast(
                    addr,
                    String::from_utf8(buf[0..n].to_vec()).unwrap(),
                ))
                .ok();
            stream.write(&buf[0..n]).unwrap();
        }
    }
    sender.send(Action::Remove(addr)).ok();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("try connecting via `telnet localhost 8080`");
    let (tx, rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();
    thread::spawn(move || loop {
        if let Ok((stream, addr)) = listener.accept() {
            {
                tx.send(Action::Add(addr, stream.try_clone().unwrap())).ok();
            }
            let thread_tx = tx.clone();
            thread::spawn(move || {
                handle_client(stream, addr, thread_tx);
            });
        }
    });
    let mut connections = Server {
        connections: HashMap::new(),
    };
    while let Ok(message) = rx.recv() {
        match message {
            Action::Add(addr, stream) => connections.add_connection(&addr, stream),
            Action::Remove(addr) => connections.remove_connection(&addr),
            Action::Broadcast(addr, msg) => connections.broadcast(&addr, msg.as_bytes()),
        }
    }
}
