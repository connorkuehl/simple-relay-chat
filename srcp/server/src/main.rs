use std::net;
use std::thread;
use std::sync::mpsc;
use std::io::Write;

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:6667").unwrap();

    let (sender, receiver) = mpsc::channel();

    let events = thread::spawn(move || {
        println!("Event thread online.");

        for received in receiver {
            println!("Event: {}", received);
        }
    });

    println!("Waiting for connections...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Incoming connection!");
        stream.write(b"Hello!\n");
        stream.flush().unwrap();

        sender.send("Message sent").unwrap();
    }

    events.join().unwrap();
}
