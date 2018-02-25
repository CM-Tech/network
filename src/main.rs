#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate piston_window;
use piston_window::*;

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::io;

enum Action {
    Add(SocketAddr, TcpStream),
    Remove(SocketAddr),
    Broadcast(String),
}

struct Server {
    connections: HashMap<SocketAddr, TcpStream>,
}

impl Server {
    fn broadcast(&mut self, msg: &String) {
        for (addr, mut connection) in self.connections.iter_mut() {
            connection.write(msg.as_bytes()).ok();
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
    }

    fn remove_connection(&mut self, addr: &SocketAddr) {
        self.connections.remove(addr);
        let msg = format!(
            "({} connections) ----- {} is disconnected -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
    }
}

fn handle_client(mut stream: TcpStream, addr: SocketAddr, sender: Sender<Action>) {
    'read: loop {
        let mut buf = [0; 4096];
        if let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break 'read;
            }
            sender
                .send(Action::Broadcast(
                    String::from_utf8(buf[0..n].to_vec()).unwrap(),
                ))
                .ok();
        }
    }
    sender.send(Action::Remove(addr)).ok();
}

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    println!("try connecting via `telnet localhost 8080`");
    println!("Would you like to \n 1) Connect to an existing socket? \n 2) Create a new socket?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let (listener, mut reader) = match &*input {
        "1\n" => {
            println!("What is the ip of the socket you would like to connect to?");
            input.clear();
            if let Ok(n) = io::stdin().read_line(&mut input) {
                input.remove(n - 1);
            }
            (None, TcpStream::connect(input + ":8080").unwrap())
        }
        "2\n" => (
            Some(TcpListener::bind("127.0.0.1:8080").unwrap()),
            TcpStream::connect("127.0.0.1:8080").unwrap(),
        ),
        _ => (None, TcpStream::connect("127.0.0.1:8080").unwrap()),
    };
    let l = listener.is_some();

    let (tx, rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();
    thread::spawn(move || loop {
        for l in listener.iter() {
            if let Ok((stream, addr)) = l.accept() {
                {
                    tx.send(Action::Add(addr, stream.try_clone().unwrap())).ok();
                }
                let thread_tx = tx.clone();
                thread::spawn(move || {
                    handle_client(stream, addr, thread_tx);
                });
            }
        }
    });

    let (reader_send, reader_read): (Sender<String>, Receiver<String>) = mpsc::channel();
    thread::spawn(move || 'read: loop {
        let mut buf = [0; 4096];
        if let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break 'read;
            }
            reader_send
                .send(String::from_utf8_lossy(&buf[0..n]).to_string())
                .ok();
        }
    });

    let mut connections = Server {
        connections: HashMap::new(),
    };
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", (640, 480))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    let mut time = 0f32;
    let mut players = vec![Point{x:50.0, y:50.0}];
    while let Some(e) = window.next() {
        if l {
            if (time % 1.0) < 1e-5 {
                connections.broadcast(&serde_json::to_string(&players[0]).unwrap());
            }
            if let Ok(message) = rx.try_recv() {
                match message {
                    Action::Add(addr, stream) => connections.add_connection(&addr, stream),
                    Action::Remove(addr) => connections.remove_connection(&addr),
                    Action::Broadcast(msg) => connections.broadcast(&msg),
                }
            }
        }
        if let Ok(message) = reader_read.try_recv() {
            players[0] = serde_json::from_str(&message).unwrap();
        }
        match e.press_args() {
            Some(Button::Keyboard(Key::L)) => players[0].x += 1.0,
            _ => (),
        };
        window.draw_2d(&e, |c, g| {
            clear(
                [
                    time.sin() / 2.0 + 0.5,
                    (time + std::f32::consts::PI / 1.5).sin() / 2.0 + 0.5,
                    (time + std::f32::consts::PI / 0.75).sin() / 2.0 + 0.5,
                    1.0,
                ],
                g,
            );
            for p in players.iter() {
                rectangle(
                    [1.0; 4],
                    [p.x - 25.0, p.y - 25.0, 50.0, 50.0],
                    c.transform,
                    g,
                )
            }
            time += 0.1;
        });
    }
}
