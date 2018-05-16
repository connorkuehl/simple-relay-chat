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

struct Command {
    sender: Arc<Client>,
    cmd_msg: String,
}

fn execute(cmd: &Command, clients: &Arc<Mutex<Vec<Client>>>) {
    let stream = cmd.sender.conn.upgrade().unwrap();

    stream.lock().unwrap().write(cmd.cmd_msg.as_bytes()).unwrap();
    stream.lock().unwrap().flush().unwrap();
}

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:6667").unwrap();

    let (sender, receiver): (mpsc::Sender<Command>, mpsc::Receiver<Command>) = mpsc::channel();

    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(vec![]));

    let allclients = Arc::downgrade(&clients);
    let events = thread::spawn(move || {
        println!("Event thread online.");

        for command in receiver {
            execute(&command, &allclients.upgrade().unwrap());
            println!("{}: {}", command.sender.user, command.cmd_msg);
        }
    });

    let pool = ThreadPool::new(NCLIENT);

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let stream = Arc::new(Mutex::new(stream));
        println!("Incoming connection!");

        let sndr = sender.clone();
        let connected = Arc::clone(&clients); 

        pool.execute(move || {
            // Mandatory identification step before able to participate in relay
            let client = Arc::new(client::identify(&stream, &connected).unwrap());
            let identified = Command {
                sender: Arc::clone(&client),
                cmd_msg: format!("/IDENTIFY {}", client.user),
            };

            sndr.send(identified).unwrap();

            loop {
                let mut message = String::new();
                if let Ok(bytes_read) = stream.lock().unwrap().read_to_string(&mut message) {
                    if bytes_read > 0 {
                        let cmd = Command {
                            sender: Arc::clone(&client),
                            cmd_msg: message,
                        };
                        sndr.send(cmd).unwrap();
                    } else {
                        break;
                    }
                } else {
                    stream.lock().unwrap().shutdown(net::Shutdown::Both).unwrap();
                    break;
                }
            }

            println!("Client disconnected.");
        });
    }

    events.join().unwrap();
    pool.join();
}
