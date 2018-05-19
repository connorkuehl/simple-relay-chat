extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync;
use std::io::{Read, Write};

const NCLIENT: usize = 32;
const MSGSIZE: usize = 1024;

struct Client;

fn handle_client(stream: net::TcpStream, event_queue: sync::mpsc::Sender<Client>) {
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
            Ok(bytes_read) => {
                if let Err(e) = event_queue.send(Client {}) {
                    eprintln!("cannot send client message to event thread: {}", e);
                    eprintln!("closing connection");
                    break;
                }

                // forward message to event thread
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
    let mut clients = vec![];

    let (sender, events_recv) = std::sync::mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");

        for event in events_recv {
            clients.push(event);
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
