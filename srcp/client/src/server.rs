use ::std;
use ::chrono;
use std::net;

use std::io::{Read, Write};

use std::collections::HashMap;

use ::common::{Command, Message};
use chrono::{TimeZone, Timelike};

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

    pub fn send(&mut self, message: &str) {
        self.conn.write(message.as_bytes()).expect("write");
        self.conn.flush().expect("flush");
    }

    pub fn update(&mut self) -> Option<()> {
        let mut buf = [0; 1024];
        match self.conn.read(&mut buf) {
            Ok(0) => {

            },
            Ok(bytes_read) => {
                let message = std::str::from_utf8(&buf[0..bytes_read])
                    .expect("from_utf8");
                let message = message.trim();

                let all_messages = message.split("\n");

                for msg in all_messages {
                    match Message::try_new(&msg) {
                        Ok(m) => {
                            let mut chathist = self.rooms.entry(m.room)
                                .or_insert(vec![]);

                            let dt = chrono::Utc.timestamp(m.time as i64, 0);
                            let human_friendly = format!("[{}:{}] {}: {}", dt.hour(), dt.minute(), m.sender, m.body);
                            chathist.push(human_friendly);
                        },
                        _ => {
                            let mut servermsgs = self.rooms.get_mut(::DEFAULT_ROOM)
                                .expect("no default room");
                            servermsgs.push(msg.to_string());
                        },
                    }

                    self.react(msg);
                }
                
                return Some(());
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return None,
                _ => (),
            },
        }
        None
    }

    fn react(&mut self, s: &str) {
        let pieces: Vec<_> = s.split_whitespace().collect();
        if pieces.len() < 5 {
            return;
        }

        let c = &pieces[4..pieces.len()].join(" ");
        let command = Command::new(c);

        match command {
            Command::Leave(room) => {
                self.rooms.remove(&room);
            },
            Command::Quit => {
                self.rooms = HashMap::new();
            }
            _ => (),
        }
    }

    pub fn get_messages(&self, room: &str) -> Option<Vec<String>> {
        if let Some(messages) = self.rooms.get(room) {
            Some(messages.clone())
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
