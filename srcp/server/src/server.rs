use ::net;
use ::std::io::Write;
use ::std::collections::HashSet;

use ::Event;
use ::Command;

pub struct Client {
    pub name: String,
    pub connection: net::TcpStream,
    pub rooms: HashSet<String>
}

pub struct Server {
    pub clients: Vec<Client>,
}

impl Server {
    pub fn new() -> Server {
        Server { clients: Vec::new() }
    }

    pub fn exec(&mut self, mut event: Event) {
        let sender_index = self.clients.iter()
            .position(|c| c.connection.peer_addr().expect("peer_addr").eq(&event.from.peer_addr().expect("peer_addr")));

        let reply = match event.command {
            Command::Identify(username) => {
                if self.clients.iter().any(|c| c.name.eq(&username)) {
                    // Respond with error that it is already taken.
                } else {
                    self.clients.push(Client {
                        name: username,
                        connection: event.from.try_clone().expect("try_clone"),
                        rooms: HashSet::new(),
                    });
                }

                event.raw
            },
            Command::Join(room) => {
                let index = sender_index.unwrap();
                self.clients[index].rooms.insert(room);

                event.raw
            },
            Command::List(room) => {
                if let Some(room) = room {
                    let subscribed_clients: Vec<String> = self.clients.iter()
                                                .filter(|c| c.rooms.iter().any(|r| r.eq(&room)))
                                                .map(|c| c.name.clone())
                                                .collect();
                    let subscribed_as_str = subscribed_clients.join(" ");

                    subscribed_as_str
                } else {
                    let size  = self.clients.iter().map(|c| &c.rooms).fold(0, |acc, v| acc + v.len() + 1);
                    let mut rooms = HashSet::new();
                    let mut rooms_as_str = String::with_capacity(size);

                    self.clients.iter().map(|c| &c.rooms).for_each(|v| {
                        for room_name in v {
                            let name = room_name.clone();
                            if rooms.insert(name) {
                                rooms_as_str.push_str(&room_name);
                                rooms_as_str.push(' ');
                            }
                        }
                    });

                    rooms_as_str
                }
            },
            Command::Say(room, message) => {
                let index = sender_index.unwrap();
                let name = self.clients[index].name.clone();
                let recipients = self.clients.iter_mut().filter(|c| c.rooms.contains(&room));

                for recipient in recipients {
                    recipient.connection.write(format!("{} {} {}\n", name, room, message).as_bytes());
                    recipient.connection.flush();
                }

                event.raw
            },
            Command::Leave(room) => {
                let cindex = sender_index.unwrap();
                self.clients[cindex].rooms.remove(&room);
                event.raw
            },
            Command::Quit => {
                if let Some(index) = self.clients.iter().position(|c| c.connection.peer_addr().unwrap().eq(&event.from.peer_addr().unwrap())) {
                    self.clients.remove(index);
                }

                event.raw
            },
            _ => String::from("unknown"),
        };

        event.from.write(format!("{}\n", reply).as_bytes());
        event.from.flush();
    }
}
