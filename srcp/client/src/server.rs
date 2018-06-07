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
        stream.set_read_timeout(Some(std::time::Duration::from_millis(85)))
            .expect("set_read_timeout");
            
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

    pub fn get_rooms(&self) -> Vec<String> {
        let r: Vec<_> = self.rooms
            .keys()
            .map(|s| s.clone())
            .collect();

        r
    }
}
