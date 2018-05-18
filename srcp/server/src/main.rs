extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Weak};
use std::io::{Read, Write};
use event::Event;
use client::Client;

mod client;
mod event;

const NCLIENT: usize = 32;

fn main() {
    let listener = net::TcpListener::bind("0.0.0.0:6667").unwrap();

    //let (sender, receiver): (mpsc::Sender<Command>, mpsc::Receiver<Command>) = mpsc::channel();
    //let (sender, receiver) = mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");
    });

    let pool = ThreadPool::new(NCLIENT);

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Incoming connection!");

        pool.execute(move || {
            loop {
                let mut message = [0; 128];
                if let Ok(bytes_read) = stream.read(&mut message) {
                    if bytes_read > 0 {
                        let msg = String::from_utf8(message.to_vec()).unwrap();
                        println!("got '{}'", msg);
                        stream.write(msg.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    } else {
                        break;
                    }
                }
            }

            println!("Client disconnected.");
        });
    }

    events.join().unwrap();
    pool.join();
}
