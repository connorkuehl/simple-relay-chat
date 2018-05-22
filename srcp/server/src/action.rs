use ::std;
use ::net;
use ::std::collections::HashSet;
use ::std::io::Write;

use ::Client;
use ::event::{Event, EventKind};

const OK: usize = 0;
const ROOM_DOESNT_EXIST: usize = 1;
/*
const USER_DOESNT_EXIST: usize = 2;
*/
const POORLY_FORMED_COMMAND: usize = 3;
const USERNAME_UNAVAILABLE: usize = 4;

pub fn execute(mut event: Event, peers: &mut Vec<Client>) {
    let (retcode, reply) = match event.kind {
        EventKind::Identify(_) => on_identify(&mut event, peers),
        EventKind::Join(_) => on_join(&mut event, peers),
        EventKind::List(_) => on_list(&mut event, peers),
        EventKind::Say(_, _) => on_say(&mut event, peers),
        EventKind::Quit => on_quit(&mut event, peers),
        EventKind::Error => (POORLY_FORMED_COMMAND, event.contents),
        _ => (999, String::from("Unknown")),
    };

    let response = format!("{} {}\n", retcode, reply);
    if let Err(e) = event.from.write(response.as_bytes()) {
        eprintln!("execute write: {}", e);
    }
    if let Err(e) = event.from.flush() {
        eprintln!("execute flush: {}", e);
    }
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
        addr: event.from.peer_addr().expect("peer_addr on identify"),
        conn: event.from.try_clone().expect("try_clone on_identify"),
        rooms: Vec::new(),
    };

    peers.push(client);

    (OK, event.contents.clone())
}

fn on_join(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    let room = match &event.kind {
        EventKind::Join(r) => r,
        _ => panic!("on_join received non-join event"),
    };

    let room = room.to_string();

    if let Some(index) = peers.iter().position(|p| p.addr.eq(&event.addr)) {
        peers[index].rooms.push(room);
    }

    (OK, event.contents.clone())
}

fn on_list(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    let to_list = match &event.kind {
        EventKind::List(option) => option,
        _ => panic!("on_list received non-list event"),
    };

    let mut retcode = OK;

    let reply = match to_list {
        Some(room) => {
            let clients: Vec<String> = peers.iter()
                .filter(|c| c.rooms.iter().any(|r| r.eq(room)))
                .map(|c| c.user.clone())
                .collect();
            if clients.len() > 0 {
                clients.join(" ")
            } else {
                retcode = ROOM_DOESNT_EXIST;
                event.contents.clone()
            }
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

    (retcode, reply)
}

fn on_say(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    let (room, message) = match &event.kind {
        EventKind::Say(r, m) => (r, m),
        _ => panic!("on_say received non-say event"),
    };

    let sender = match peers.iter().position(|p| p.addr.eq(&event.addr)) {
        Some(index) => peers[index].user.clone(),
        None => String::from("unidentified"),
    };

    let time = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(t) => t.as_secs(),
        Err(_) => 0,
    };

    let final_msg = format!("{} {} {} {} {}\n", OK, sender, time, room, message);

    let mut send_to = peers.into_iter()
        .filter(|p| p.rooms.iter().any(|r| r.eq(room)))
        .peekable();

    if send_to.peek().is_none() {
        return (ROOM_DOESNT_EXIST, event.contents.clone());
    }

    for client in send_to {
        client.conn.write(final_msg.as_bytes()).expect("write");
        client.conn.flush().expect("flush");
    }

    (OK, event.contents.clone())
}

fn on_quit(event: &mut Event, peers: &mut Vec<Client>) -> (usize, String) {
    // send message to subscribed channels saying they left
    // probably just call on_leave for each of them.

    if let Some(index) = peers.iter().position(|p| p.addr.eq(&event.addr)) {
        peers.remove(index);
    }

    match event.from.shutdown(net::Shutdown::Both) {
        _ => (),
    }

    (OK, event.contents.clone())
}
