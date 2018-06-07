use ::std;
use std::net;

pub struct Server {
    conn: net::TcpStream,
}

impl Server {
    pub fn new(addr: &str) -> std::io::Result<Server> {
        let stream = net::TcpStream::connect(addr)?;

        Ok(Server {
            conn: stream,
        })
    }
}
