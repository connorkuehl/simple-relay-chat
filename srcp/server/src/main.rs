extern crate common;

extern crate threadpool;

use threadpool::ThreadPool;

use std::net;
use std::thread;
use std::sync;
use std::io::Read;

use server::Server;

use common::Command;
mod server;

// Max number of supported clients for the server.
const NCLIENT: usize = 32;
const MSGSIZE: usize = 1024;

pub struct Event {
    from: net::TcpStream,
    command: Command,
    raw: String,
}

// Entry point for client threads. Listens for message from
// the client, parses it, and sends to the event processing
// thread for relaying the message.
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
                let message = std::str::from_utf8(&buf);

                if message.is_err() {
                    continue;
                }

                let message = message.unwrap();

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

    let quit = Event {
        from: stream.try_clone().expect("try_clone on client quit"),
        command: Command::Quit,
        raw: "QUIT".to_string(),
    };

    if let Err(e) = cmd_queue.send(quit) {
        eprintln!("cannot send client quit to event thread: {}", e);
    }

    match stream.shutdown(net::Shutdown::Both) {
        _ => println!("Disconnected from {}.", remote),
    }
}

fn main() {
    let listener = net::TcpListener::bind("0.0.0.0:6667").expect("bind");

    let (sender, command_queue) = std::sync::mpsc::channel();

    // Event Processing Thread: executes parsed commands
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
