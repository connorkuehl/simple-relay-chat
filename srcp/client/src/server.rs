use ::std;
use std::net;

use std::collections::HashMap;

pub struct Server {
    conn: net::TcpStream,
    rooms: HashMap<String, Vec<String>>,
}

impl Server {
    pub fn new(addr: &str) -> std::io::Result<Server> {
        let stream = net::TcpStream::connect(addr)?;
        let mut r = HashMap::new();

        r.insert(String::from(::DEFAULT_ROOM), vec![]);

        Ok( Server { conn: stream, rooms: r, } )
    }

    pub fn get_messages(&self, room: &str) -> Option<&[String]> {
        if let Some(messages) = self.rooms.get(room) {
            Some(messages.as_slice())
        } else {
            None
        }
    }
}
