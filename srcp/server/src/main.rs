extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync::mpsc;
use std::io::Read;
use event::Event;

mod client;
mod event;

const NCLIENT: usize = 32;

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:6667").unwrap();

    let (sender, receiver) = mpsc::channel();

    let events = thread::spawn(move || {
        let mut clients = vec![];
        println!("Event thread online.");

        for received in receiver {
            match received {
                Event::Identify(client) => {
                    clients.push(client);
                },
                _ => (),
            }
        }
    });

    let pool = ThreadPool::new(NCLIENT);

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Incoming connection!");

        let sndr = sender.clone();
        pool.execute(move || {
            if let Ok(client) = client::identify(&mut stream) {
                sndr.send(Event::Identify(client)).unwrap();
            } else {
                stream.shutdown(net::Shutdown::Both).unwrap();
            }

            loop {
                let mut message = String::new();
                if let Ok(bytes_read) = stream.read_to_string(&mut message) {
                    if bytes_read > 0 {
                        sndr.send(event::parse(message).unwrap()).unwrap();
                    } else {
                        break;
                    }
                } else {
                    stream.shutdown(net::Shutdown::Both).unwrap();
                    break;
                }
            }

            println!("Client disconnected.");
        });
    }

    events.join().unwrap();
    pool.join();
}
