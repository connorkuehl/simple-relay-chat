use ::net;
use ::std::time;
use ::std::io::Write;
use ::std::collections::{HashSet, HashMap};

use ::Event;
use ::common::{Command, StatusCode};

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
        let (code, resp) = match event.command {
            Command::Identify(username) => {
                if self.clients.iter().any(|c| c.name.eq(&username)) {
                    // Respond with error that it is already taken.
                    (StatusCode::UsernameUnavailable, event.raw)
                } else {
                    self.clients.push(Client {
                        name: username,
                        connection: event.from.try_clone().expect("try_clone"),
                        rooms: HashSet::new(),
                    });

                    (StatusCode::Ok, event.raw)
                }
            },
            _ => { 
                // These commands may only be invoked after a client has identified
                // themselves.

                // This index refers to the SENDING CLIENT's position index in the
                // `clients` Vec
                let index = assert_identified!(self.clients, event);
                let sender_name = self.clients[index].name.clone();
                
                match event.command {
                    // Joins a room or creates one if it doesn't yet exist.
                    Command::Join(room) => {
                        self.clients[index].rooms.insert(room.clone());

                        let list = self.rooms.entry(room.clone()).or_insert(vec![]);

                        if list.iter().any(|c| c.name.eq(&sender_name)) {
                            (StatusCode::AlreadyJoined, event.raw)
                        } else {
                            list.push(self.clients[index].clone());

                            // Announce that this client has joined.
                            let joinmsg = Server::create_message(
                                0, 
                                &format!("{} has joined.", self.clients[index].name), 
                                "server", 
                                &room
                            );
                            Server::say(list.as_mut_slice(), &joinmsg);

                            (StatusCode::Ok, event.raw)
                        }
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
                                    (StatusCode::Ok, usernames.join(" "))
                                },
                                None => {
                                    // Error
                                    (StatusCode::RoomDoesntExist, event.raw)
                                },
                            }
                        } else {
                            // User did not provide a room name, so list all the rooms on the server.
                            let rooms: Vec<String> = self.rooms.keys().map(|k| k.clone()).collect();
                            (StatusCode::Ok, rooms.join(" "))
                        }
                    },
                    // Sends a message to a room.
                    Command::Say(room, message) => {
                        self.on_say(&room, &sender_name, &message);

                        (StatusCode::Ok, event.raw)
                    },
                    // Sends a private message to a connected client.
                    Command::Whisper(to, message) => {
                        let rc = match self.clients.iter().position(|c| c.name.eq(&to)) {
                            Some(index) => {
                                let recipient = self.clients[index].clone();
                                let message = Server::create_message(0, &message, &sender_name, &to);
                                Server::say(&mut[recipient], &message);

                                StatusCode::Ok
                            },
                            None => StatusCode::UserDoesntExist, 
                        };
                        
                        (rc, event.raw)
                    },
                    // Broadcasts a message to all rooms.
                    Command::Shout(message) => {
                        let rooms: Vec<_> = self.rooms.iter().map(|(r, _)| r.clone()).collect();

                        for room in rooms {
                            self.on_say(&room, &sender_name, &message);
                        }

                        (StatusCode::Ok, event.raw)
                    },
                    // Leaves a room.
                    Command::Leave(room) => {
                        self.on_leave(&room, &sender_name, index);

                        (StatusCode::Ok, event.raw)
                    },
                    // Disconnects from the server; as a consequence, leaves all
                    // rooms, too.
                    Command::Quit => {
                        let client = self.clients[index].clone();

                        // unsubscribe them from each room they belong to.
                        let subscribed: Vec<_> = client.rooms.iter().map(|r| r.clone()).collect();
                        for room in subscribed {
                            self.on_leave(&room, &sender_name, index);
                        }

                        ignore_result(client.connection.shutdown(net::Shutdown::Both));

                        // remove from list of clients
                        self.clients.remove(index);

                        (StatusCode::Ok, event.raw)
                    },
                    _ => (StatusCode::PoorlyFormedCommand, event.raw),
                }
            }
        };

        // Echo the command that was just processed back to the client.
        let reply = Server::create_message(code as usize, &resp, "server", "server");

        Server::say(
            &mut [Client { 
                name: String::from("repl"), 
                connection: event.from.try_clone().expect("try_clone"), 
                rooms: HashSet::new()}
                ], 
                &reply
        );
    }

    fn on_say(&mut self, room: &str, user: &str, message: &str) {
        if let Some(recipients) = self.rooms.get_mut(room) {
            let message = Server::create_message(0, message, user, room);
            Server::say(recipients.as_mut_slice(), &message);
        }
    }

    // Gracefully unsubscribes user from the room.
    fn on_leave(&mut self, room: &str, user: &str, index: usize) {
        // If the client is subscribed to the room
        if self.clients[index].rooms.contains(room) {
            // If the room actually exists
            if let Some(subscribed) = self.rooms.get_mut(&room.to_string()) {
                // Announce that the user is leaving.
                let message = Server::create_message(0, &format!("{} has left.", user), "server", room);
                Server::say(subscribed.as_mut_slice(), &message);

                if let Some(cindex) = subscribed.iter().position(|c| c.name.eq(&user)) {
                    subscribed.remove(cindex);
                }
            }

            // Clear out the empty rooms.
            self.clients[index].rooms.remove(room);
            let empties: Vec<_> = self.rooms
                .iter()
                .filter(|&(_, ref v)| v.len() == 0)
                .map(|(k, _)| k.clone())
                .collect();
            for empty in empties {
                self.rooms.remove(&empty);
            }
        }
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
