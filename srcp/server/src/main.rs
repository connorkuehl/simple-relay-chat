extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync;
use std::io::Read;

use event::Event;

const NCLIENT: usize = 32;
const MSGSIZE: usize = 1024;

mod action;
mod event;

pub struct Client {
    pub user: String,
    pub addr: net::SocketAddr,
    pub conn: net::TcpStream,
    pub rooms: Vec<String>,
}

fn parse_message(s: &str, from: &net::TcpStream) -> Event {
    Event {
        from: from.try_clone().expect("parse_message: try_clone"),
        addr: from.peer_addr().expect("parse_message: peer_addr"),
        kind: event::kind_parse(s),
        contents: String::from(s),
    }
}

fn handle_client(mut stream: net::TcpStream, event_queue: sync::mpsc::Sender<Event>) {
    let remote = stream.peer_addr().expect("peer_addr");

    println!("{} has connected.", remote);

    loop {
        let mut buf = [0; MSGSIZE];
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("{} has disconnected.", remote);
                break;
            },
            Ok(bytes_read) => {
                let message = std::str::from_utf8(&buf).expect("from utf8");

                let event = parse_message(&message[..bytes_read], &stream);

                if let Err(e) = event_queue.send(event) {
                    eprintln!("cannot send client message to event thread: {}", e);
                    eprintln!("closing connection");
                    break;
                }
            },
            Err(_) => break,
        }
    }

    match stream.shutdown(net::Shutdown::Both) {
        _ => println!("Disconnected from {}.", remote),
    }
}

fn main() {
    let mut clients = vec![];
    let listener = net::TcpListener::bind("0.0.0.0:6667").expect("bind");

    let (sender, events_recv) = std::sync::mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");

        for event in events_recv {
            let mut event = event;
            action::execute(event, &mut clients);
        }
    });

    let pool = ThreadPool::new(NCLIENT);

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            println!("Incoming connection!");
            let events_queue = sender.clone();
            pool.execute(move || {
                handle_client(stream, events_queue);
            });
        } else {
            eprintln!("Failed to accept incoming connection.");
        }
    }

    events.join().expect("events join");
    pool.join();
}
