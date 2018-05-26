use ::net;
use ::std::time;
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

    pub fn exec(&mut self, event: Event) {
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
                self.clients[index].rooms.insert(room.clone());
                let mut others: Vec<net::TcpStream> = self.clients.iter()
                                                    .filter(|c| c.rooms.iter().any(|r| r.eq(&room)))
                                                    .map(|c| c.connection.try_clone().expect("try_clone"))
                                                    .collect();

                let joinmsg = Server::create_message(0, &format!("{} has joined.", self.clients[index].name), "server", &room);
                self.say(others.as_mut_slice(), &joinmsg);



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
                let mut recipients: Vec<net::TcpStream> = self.clients
                    .iter()
                    .filter(|c| c.rooms.contains(&room))
                    .map(|c| c.connection.try_clone().expect("try_clone"))
                    .collect();

                let message = Server::create_message(0, &message, &name, &room);
                self.say(recipients.as_mut_slice(), &message);

                event.raw
            },
            Command::Leave(room) => {
                let cindex = sender_index.unwrap();
                self.clients[cindex].rooms.remove(&room);

                let mut others: Vec<net::TcpStream> = self.clients.iter()
                    .filter(|c| c.rooms.contains(&room))
                    .map(|c| c.connection.try_clone().expect("try_clone"))
                    .collect();

                let message = format!("{} has left the room.", self.clients[cindex].name);
                let message = Server::create_message(0, &message, "server", &room);
                self.say(others.as_mut_slice(), &message);

                event.raw
            },
            Command::Quit => {
                if let Some(index) = self.clients.iter().position(|c| c.connection.peer_addr().unwrap().eq(&event.from.peer_addr().unwrap())) {
                    let cindex = sender_index.unwrap();

                    for subscribed in self.clients[index].rooms.clone() {
                        let mut others: Vec<net::TcpStream> = self.clients.iter()
                            .filter(|c| c.rooms.contains(&subscribed))
                            .map(|c| c.connection.try_clone().expect("try_clone"))
                            .collect();

                        let message = format!("{} has left the room.", self.clients[cindex].name);
                        let message = Server::create_message(0, &message, "server", &subscribed);
                        self.say(others.as_mut_slice(), &message);
                    }

                    self.clients.remove(index);
                }

                event.raw
            },
            _ => String::from("unknown"),
        };

        let reply = Server::create_message(0, &reply, "server", "server");
        self.say(&mut [event.from], &reply);
    }

    fn say(&mut self, to: &mut[net::TcpStream], what: &str) {
        for client in to {
            ignore_result(client.write(format!("{}", what).as_bytes()));
            ignore_result(client.flush());
        }
    }

    fn create_message(code: usize, body: &str, from: &str, to_room: &str) -> String {
        let time = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            _ => 0,
        };

        format!("{} {} {} {} {}\n", code, from, time, to_room, body)
    }
}

fn ignore_result<R, E>(r: Result<R, E>) {
    match r {
        _ => (),
    }
}
