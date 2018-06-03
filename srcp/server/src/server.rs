use ::net;
use ::std::time;
use ::std::io::Write;
use ::std::collections::{HashSet, HashMap};

use ::Event;
use ::Command;

// Cancels event execution and shuts down the connection
// if the invoking client has not identified themselves.
// 
// This is a macro so we can return early from the server
// exec function.
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

    // Executes a command received by a client thread.
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
                // These commands may only be invoked after a client has identified
                // themselves.

                // This index refers to the SENDING CLIENT's position index in the
                // `clients` Vec
                let index = assert_identified!(self.clients, event);
                match event.command {
                    // Joins a room or creates one if it doesn't yet exist.
                    Command::Join(room) => {
                        self.clients[index].rooms.insert(room.clone());

                        let list = self.rooms.entry(room.clone()).or_insert(vec![]);
                        list.push(self.clients[index].clone());

                        // Announce that this client has joined.
                        let joinmsg = Server::create_message(
                            0, 
                            &format!("{} has joined.", self.clients[index].name), 
                            "server", 
                            &room
                        );
                        Server::say(list.as_mut_slice(), &joinmsg);

                        event.raw
                    },
                    // Lists all rooms or lists the people in that room depending on if
                    // an Option argument is given.
                    Command::List(room) => {
                        // User provided a room name, so list the people inside.
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
                            // User did not provide a room name, so list all the rooms on the server.
                            let rooms: Vec<String> = self.rooms.keys().map(|k| k.clone()).collect();
                            rooms.join(" ")
                        }
                    },
                    // Sends a message to a room.
                    Command::Say(room, message) => {
                        let name = self.clients[index].name.clone();
                        if let Some(recipients) = self.rooms.get_mut(&room) {
                            let message = Server::create_message(0, &message, &name, &room);
                            Server::say(recipients.as_mut_slice(), &message);
                        }

                        event.raw
                    },
                    // Leaves a room.
                    Command::Leave(room) => {
                        let user = self.clients[index].name.clone();

                        // If the client is subscribed to the room
                        if self.clients[index].rooms.contains(&room) {
                            // If the room actually exists
                            if let Some(subscribed) = self.rooms.get_mut(&room) {
                                // Announce that the user is leaving.
                                let message = Server::create_message(0, &format!("{} has left.", user), "server", &room);
                                Server::say(subscribed.as_mut_slice(), &message);

                                if let Some(cindex) = subscribed.iter().position(|c| c.name.eq(&user)) {
                                    subscribed.remove(cindex);
                                }
                            }
                            
                            self.clients[index].rooms.remove(&room);
                            let empties: Vec<_> = self.rooms
                                .iter()
                                .filter(|&(_, ref v)| v.len() == 0)
                                .map(|(k, _)| k.clone())
                                .collect();
                            for empty in empties {
                                self.rooms.remove(&empty);
                            }
                        }

                        event.raw
                    },
                    // Disconnects from the server; as a consequence, leaves all
                    // rooms, too.
                    Command::Quit => {
                        let client = self.clients[index].clone();
                        let user = client.name.clone();

                        // Gracefully unsubscribe user from all connected rooms
                        for room in client.rooms {
                            if let Some(subscribed) = self.rooms.get_mut(&room) {
                                // room exists, notify clients that user is leaving
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

        // Echo the command that was just processed back to the client.
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

    // Sends a message to specified clients.
    fn say(to: &mut[Client], what: &str) {
        for client in to {
            ignore_result(client.connection.write(format!("{}", what).as_bytes()));
            ignore_result(client.connection.flush());
        }
    }

    // Creates a formatted message
    // <opcode> <sender> <timestamp> <room> <message>
    fn create_message(code: usize, body: &str, from: &str, to_room: &str) -> String {
        let time = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            _ => 0,
        };

        format!("{} {} {} {} {}\n", code, from, time, to_room, body)
    }
}

// Reckless utility function; there are times where
// I am just making sure something has been shut down
// and don't care if it has already been shut down.
fn ignore_result<R, E>(r: Result<R, E>) {
    match r {
        _ => (),
    }
}
