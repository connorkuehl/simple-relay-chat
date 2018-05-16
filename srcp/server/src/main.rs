extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync::mpsc;
use std::io::Read;

const NCLIENT: usize = 32;

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:6667").unwrap();

    let (sender, receiver) = mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");

        for received in receiver {
            println!("Event: {}", received);
        }
    });

    let pool = ThreadPool::new(NCLIENT);

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Incoming connection!");

        let sndr = sender.clone();
        pool.execute(move || {
            loop {
                let mut message = String::new();
                if let Ok(bytes_read) = stream.read_to_string(&mut message) {
                    if bytes_read > 0 {
                        sndr.send(message).unwrap();
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
