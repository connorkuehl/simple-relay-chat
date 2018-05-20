use ::net;
use ::std::collections::HashSet;
use ::std::io::Write;

use ::Client;
use ::event::{Event, EventKind};

const OK: usize = 0;
const ROOM_DOESNT_EXIST: usize = 1;
const USER_DOESNT_EXIST: usize = 2;
const POORLY_FORMED_COMMAND: usize = 3;
const USERNAME_UNAVAILABLE: usize = 4;

pub fn execute(event: Event, peers: &mut Vec<Client>) {
    let mut event = event;

    let (retcode, reply) = match event.kind {
        EventKind::Identify(_) => on_identify(&mut event, peers),
        EventKind::List(_) => on_list(&mut event, peers),
        EventKind::Quit => on_quit(&mut event, peers),
        _ => (999, String::from("Unknown")),
    };

    let response = format!("{} {}", retcode, reply);
    event.from.write(response.as_bytes());
    event.from.flush();
}

fn on_identify(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    let username = match &event.kind {
        EventKind::Identify(user) => user,
        _ => panic!("on_identify received non-identify event"),
    };

    if peers.iter().any(|p| p.user.eq(username)) {
        return (USERNAME_UNAVAILABLE, event.contents.clone());
    }

    let client = Client {
        user: username.to_string(),
        conn: event.from.try_clone().expect("try_clone on_identify"),
        rooms: Vec::new(),
    };

    peers.push(client);

    (OK, event.contents.clone())
}

fn on_list(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    let to_list = match &event.kind {
        EventKind::List(option) => option,
        _ => panic!("on_list received non-list event"),
    };

    let reply = match to_list {
        Some(room) => {
            let clients: Vec<String> = peers.iter()
                .filter(|c| c.rooms.iter().any(|r| r.eq(room)))
                .map(|c| c.user.clone())
                .collect();
            clients.join(" ")
        },
        None => {
            let size = peers.iter().map(|p| &p.rooms).fold(0, |acc, v| acc + v.len() + 1);
            let mut rooms = HashSet::new();
            let mut string = String::with_capacity(size);

            peers.iter().map(|p| &p.rooms).for_each(|v| {
                for room_name in v {
                    let name = room_name.clone();
                    if rooms.insert(name) {
                        string.push_str(&room_name);
                        string.push(' ');
                    }
                }
            });

            string
        },
    };

    (OK, reply)
}

fn on_quit(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    // send message to subscribed channels saying they left
    // probably just call on_leave for each of them.

    let addr = event.from.peer_addr().expect("peer_addr");
    if let Some(index) = peers.iter().position(|x| x.conn.peer_addr().expect("peer_addr").eq(&addr)) {
        peers.remove(index);
    }

    match event.from.shutdown(net::Shutdown::Both) {
        _ => (),
    }

    (OK, event.contents.clone())
}
