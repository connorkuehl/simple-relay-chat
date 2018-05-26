use ::net;
use ::std::time;
use ::std::io::Write;
use ::std::collections::{HashSet, HashMap};

use ::Event;
use ::Command;

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
                let index = sender_index.unwrap();
                let name = self.clients[index].name.clone();
                if let Some(recipients) = self.rooms.get_mut(&room) {
                    let message = Server::create_message(0, &message, &name, &room);
                    Server::say(recipients.as_mut_slice(), &message);
                }

                event.raw
            },
            Command::Leave(room) => {
                event.raw
            },
            Command::Quit => {
                if let Some(index) = self.clients.iter().position(|c| c.connection.peer_addr().unwrap().eq(&event.from.peer_addr().unwrap())) {
                    let cindex = sender_index.unwrap();

                    self.clients.remove(index);
                }

                event.raw
            },
            _ => String::from("unknown"),
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
