use ::std;
use std::net;

use std::collections::HashMap;
use std::collections::VecDeque;

pub struct Server {
    conn: net::TcpStream,
    rooms: HashMap<String, VecDeque<String>>,
}

impl Server {
    pub fn new(addr: &str) -> std::io::Result<Server> {
        let stream = net::TcpStream::connect(addr)?;
        let mut r = HashMap::new();

        r.insert(String::from(::DEFAULT_ROOM), VecDeque::new());

        Ok( Server { conn: stream, rooms: r, } )
    }
}
