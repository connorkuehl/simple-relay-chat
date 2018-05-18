extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::io::{Read, Write};

mod client;
mod event;

const NCLIENT: usize = 32;
const MSGSIZE: usize = 1024;

fn main() {
    let listener = net::TcpListener::bind("0.0.0.0:6667").expect("bind");

    //let (sender, receiver): (mpsc::Sender<Command>, mpsc::Receiver<Command>) = mpsc::channel();
    //let (sender, receiver) = mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");
    });

    let pool = ThreadPool::new(NCLIENT);

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        let mut stream = stream.expect("incoming");
        println!("Incoming connection!");

        pool.execute(move || {
            loop {
                let mut message = [0; MSGSIZE];
                if let Ok(bytes_read) = stream.read(&mut message) {
                    if bytes_read > 0 {
                        let msg = message.to_vec();
                        let msg = String::from_utf8_lossy(&msg);
                        println!("got '{}'", msg);
                        stream.write(&message).expect("echo");
                        stream.flush().expect("echo flush");
                    } else {
                        break;
                    }
                }
            }

            println!("Client disconnected.");
        });
    }

    events.join().expect("events join");
    pool.join();
}
