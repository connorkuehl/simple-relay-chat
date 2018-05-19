extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync;
use std::io::{Read, Write};

use event::{Event, EventKind};

const NCLIENT: usize = 32;
const MSGSIZE: usize = 1024;

mod event;

fn parse_message(s: String, from: &net::TcpStream) -> Event {
    Event {
        from: from.try_clone().expect("try_clone"),
        kind: event::kind_parse(&s),
    }
}

fn handle_client(stream: net::TcpStream, event_queue: sync::mpsc::Sender<Event>) {
    let mut stream = stream;
    let remote = stream.peer_addr().expect("peer_addr");

    println!("{} has connected.", remote);

    loop {
        let mut message = [0; MSGSIZE];
        match stream.read(&mut message) {
            Ok(0) => {
                println!("{} has disconnected.", remote);
                break;
            },
            Ok(_bytes_read) => {
                let message = message.to_vec();
                let message = String::from_utf8_lossy(&message).into_owned();

                let event = parse_message(message, &stream);

                if let Err(e) = event_queue.send(event) {
                    eprintln!("cannot send client message to event thread: {}", e);
                    eprintln!("closing connection");
                    break;
                }
            },
            Err(_) => break,
        }
    }

    if let Ok(_) = stream.shutdown(net::Shutdown::Both) {
        println!("Disconnected from {}.", remote);
    } 
}

fn main() {
    let listener = net::TcpListener::bind("0.0.0.0:6667").expect("bind");

    let (sender, events_recv): (std::sync::mpsc::Sender<Event>, std::sync::mpsc::Receiver<Event>) = std::sync::mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");

        for event in events_recv {
            let mut event = event;
            match event.kind {
                EventKind::Identify(user) => {
                    event.from.write(user.as_bytes()).unwrap();
                    event.from.flush().unwrap();
                },
                EventKind::Error(err) => {
                    event.from.write(err.as_bytes()).unwrap();
                    event.from.flush().unwrap();
                },
            }
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
