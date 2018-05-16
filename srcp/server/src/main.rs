use std::net;
use std::io::Write;

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:6667").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Incoming connection!");
        stream.write(b"Hello!\n");
        stream.flush().unwrap();
    }
}
