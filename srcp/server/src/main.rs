extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync;
use std::io::Read;

use command::Command;
use server::Server;

mod command;
mod server;

const NCLIENT: usize = 32;
const MSGSIZE: usize = 1024;

#[derive(Debug)]
pub struct Event {
    from: net::TcpStream,
    command: Command,
    raw: String,
}

fn handle_client(mut stream: net::TcpStream, cmd_queue: sync::mpsc::Sender<Event>) {
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

                let event = Event {
                    from: stream.try_clone().expect("try_clone on client thread"),
                    command: Command::new(&message[0..bytes_read]),
                    raw: message[0..bytes_read].trim().to_string(),
                };

                if let Err(e) = cmd_queue.send(event) {
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
    let listener = net::TcpListener::bind("0.0.0.0:6667").expect("bind");

    let (sender, command_queue) = std::sync::mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");
        let mut server = Server::new();
        for cmd in command_queue {
            server.exec(cmd);
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
