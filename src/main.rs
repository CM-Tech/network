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

fn handle_client(mut stream: TcpStream, addr: SocketAddr, sender: Sender<Action>) {
    stream.write(b"testing\n").unwrap();
    stream.flush().unwrap();
    loop {
        let mut buf = [0; 4096];
        if let Ok(n) = stream.read(&mut buf) {
            sender.send(Action::Broadcast(
                addr,
                String::from_utf8(buf[0..n].to_vec()).unwrap(),
            ));
            stream.write(&buf[0..n]).unwrap();
        }
    }
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
    let mut connections = HashMap::new();
    while let Ok(message) = rx.recv() {
        match message {
            Action::Add(addr, stream) => add_connection(&mut connections, &addr, stream),
            Action::Remove(addr) => remove_connection(&mut connections, &addr),
            Action::Broadcast(addr, msg) => broadcast(&mut connections, &addr, msg.as_bytes()),
        }
        fn broadcast(
            connections: &mut HashMap<SocketAddr, Connection>,
            from: &SocketAddr,
            msg: &[u8],
        ) {
            println!(
                "broadcasting msg: {}",
                String::from_utf8(msg.to_vec()).unwrap()
            );
            for (addr, mut connection) in connections.iter_mut() {
                if *from == *addr {
                    continue;
                }
                connection.stream.write(msg).ok();
                connection.stream.flush().ok();
            }
        }

        fn add_connection(
            connections: &mut HashMap<SocketAddr, Connection>,
            addr: &SocketAddr,
            stream: TcpStream,
        ) {
            connections.insert(
                *addr,
                Connection {
                    addr: *addr,
                    stream: stream,
                },
            );
            let msg = format!(
                "({} connections) ----- new connection from {} -----",
                connections.len(),
                addr
            );
            println!("{}", msg);
            broadcast(connections, addr, (msg + "\n").as_bytes());
        }

        fn remove_connection(connections: &mut HashMap<SocketAddr, Connection>, addr: &SocketAddr) {
            connections.remove(addr);
            let msg = format!(
                "({} connections) ----- {} is disconnected -----",
                connections.len(),
                addr
            );
            println!("{}", msg);
            broadcast(connections, addr, (msg + "\n").as_bytes());
        }
    }
}
