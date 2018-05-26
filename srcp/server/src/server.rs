use ::net;
use ::std::time;
use ::std::io::Write;
use ::std::collections::{HashSet, HashMap};

use ::Event;
use ::Command;

macro_rules! assert_identified {
    ( $x: expr, $y: ident ) => {
        {
            let temp_index = $x.iter()
                .position(|c| c.connection.peer_addr().expect("peer_addr").eq(&$y.from.peer_addr().expect("peer_addr")));

            if temp_index.is_none() {
                ignore_result($y.from.write(&format!("9 {}\n", "UNIDENTIFIED").as_bytes()));
                ignore_result($y.from.flush());
                ignore_result($y.from.shutdown(net::Shutdown::Read));
                return;
            } else {
                temp_index.unwrap()
            }
        }
    };
}

pub struct Client {
    pub name: String,
    pub connection: net::TcpStream,
    pub rooms: HashSet<String>
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            name: self.name.clone(),
            connection: self.connection.try_clone().expect("try_clone"),
            rooms: self.rooms.clone(),
        }
    }
}

pub struct Server {
    pub clients: Vec<Client>,
    pub rooms: HashMap<String, Vec<Client>>,
}

impl Server {
    pub fn new() -> Server {
        Server { 
            clients: Vec::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn exec(&mut self, mut event: Event) {
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
            _ => { 
                let index = assert_identified!(self.clients, event);
                match event.command {
                    Command::Join(room) => {
                        self.clients[index].rooms.insert(room.clone());

                        let list = self.rooms.entry(room.clone()).or_insert(vec![]);
                        list.push(self.clients[index].clone());

                        let joinmsg = Server::create_message(
                            0, 
                            &format!("{} has joined.", self.clients[index].name), 
                            "server", 
                            &room
                        );
                        Server::say(list.as_mut_slice(), &joinmsg);

                        event.raw
                    },
                    Command::List(room) => {
                        // user provided a room name
                        if let Some(room) = room {
                            match self.rooms.get(&room) {
                                // room exists
                                Some(rm) => {
                                    let usernames: Vec<String> = rm.iter().map(|c| c.name.clone()).collect();
                                    usernames.join(" ")
                                },
                                None => {
                                    // Error
                                    String::from("room doesn't exist")
                                },
                            }
                        } else {
                            let rooms: Vec<String> = self.rooms.keys().map(|k| k.clone()).collect();
                            rooms.join(" ")
                        }
                    },
                    Command::Say(room, message) => {
                        let name = self.clients[index].name.clone();
                        if let Some(recipients) = self.rooms.get_mut(&room) {
                            let message = Server::create_message(0, &message, &name, &room);
                            Server::say(recipients.as_mut_slice(), &message);
                        }

                        event.raw
                    },
                    Command::Leave(room) => {
                        let user = self.clients[index].name.clone();

                        if self.clients[index].rooms.contains(&room) {
                            if let Some(subscribed) = self.rooms.get_mut(&room) {
                                let message = Server::create_message(0, &format!("{} has left.", user), "server", &room);
                                Server::say(subscribed.as_mut_slice(), &message);
                            }

                            self.clients[index].rooms.remove(&room);
                        }

                        event.raw
                    },
                    Command::Quit => {
                        let client = self.clients[index].clone();
                        let user = client.name.clone();

                        for room in client.rooms {
                            if let Some(subscribed) = self.rooms.get_mut(&room) {
                                // channel exists, notify clients that user is leaving
                                let message = Server::create_message(0, &format!("{} has left.", &user), "server", &room);
                                Server::say(subscribed.as_mut_slice(), &message);

                                // remove this user from the room's list of subscribed clients
                                if let Some(pos) = subscribed.iter().position(|c| c.name.eq(&user)) {
                                    subscribed.remove(pos);
                                }
                            }
                        }

                        ignore_result(client.connection.shutdown(net::Shutdown::Both));

                        // remove from list of clients
                        self.clients.remove(index);

                        event.raw
                    },
                    _ => String::from("unknown"),
                }
            }
        };

        let reply = Server::create_message(0, &reply, "server", "server");

        Server::say(
            &mut [Client { 
                name: String::from("repl"), 
                connection: event.from.try_clone().expect("try_clone"), 
                rooms: HashSet::new()}
                ], 
                &reply
        );
    }

    fn say(to: &mut[Client], what: &str) {
        for client in to {
            ignore_result(client.connection.write(format!("{}", what).as_bytes()));
            ignore_result(client.connection.flush());
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
