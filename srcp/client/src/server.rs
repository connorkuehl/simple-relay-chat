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

        Ok(Server {
            conn: stream,
            rooms: HashMap::new(),
        })
    }
}
