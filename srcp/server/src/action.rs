use ::std;
use ::net;
use ::std::collections::HashSet;
use ::std::io::Write;

use ::Client;
use ::event::{Event, EventKind};

#[derive(Clone, Copy)]
enum ResponseCode {
    OK = 0,
    ROOM_DOESNT_EXIST = 1,
    USER_DOESNT_EXIST = 2,
    POORLY_FORMED_COMMAND = 3,
    USERNAME_UNAVAILABLE = 4,
}

pub fn execute(mut event: Event, peers: &mut Vec<Client>) {
    let retcode = match event.kind {
        EventKind::Identify(_) => on_identify(&mut event, peers),
        EventKind::Join(_) => on_join(&mut event, peers),
        EventKind::Leave(_) => on_leave(&mut event, peers),
        EventKind::List(_) => on_list(&mut event, peers),
        EventKind::Say(_, _) => on_say(&mut event, peers),
        EventKind::Quit => on_quit(&mut event, peers),
        EventKind::Error => ResponseCode::POORLY_FORMED_COMMAND,
    };

    let response = format!("{} {}\n", retcode as usize, event.contents);
    write_and_ignore(&mut event.from, &response)
}

fn write_and_ignore(connection: &mut net::TcpStream, what: &str) {
    match connection.write(what.as_bytes()) {
        _ => (),
    }

    match connection.flush() {
        _ => (),
    }
}

fn create_message(what: &str, room: &str, from: &str) -> String {
    let time = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(t) => t.as_secs(),
        Err(_) => 0,
    };

    format!("{} {} {} {}\n", from, time, room, what)
}

fn server_say_to_room(what: &str, room: &str, from: &str, peers: &mut Vec<Client>) {
    let recipients = peers.into_iter()
        .filter(|p| p.is_subscribed(room));


    let message = format!("{} {}", ResponseCode::OK as usize, create_message(what, room, from));

    for recipient in recipients {
        write_and_ignore(&mut recipient.conn, &message);
    }
}

fn server_say_to(err: ResponseCode, what: &str, to: &str, peers: &mut Vec<Client>) {
    if let Some(to) = peers.iter().position(|p| p.user.eq(to)) {
        let message = format!("{} {}", err as usize, create_message(what, "server", "server"));
        write_and_ignore(&mut peers[to].conn, &message);
    }
}

fn on_identify(event: &mut Event, peers: &mut Vec<Client>) -> ResponseCode {
    let username = match &event.kind {
        EventKind::Identify(user) => user,
        _ => panic!("on_identify received non-identify event"),
    };

    if peers.iter().any(|p| p.user.eq(username)) {
        return ResponseCode::USERNAME_UNAVAILABLE;
    }

    let client = Client {
        user: username.to_string(),
        addr: event.from.peer_addr().expect("peer_addr on identify"),
        conn: event.from.try_clone().expect("try_clone on_identify"),
        rooms: Vec::new(),
    };

    peers.push(client);

    ResponseCode::OK
}

fn on_join(event: &mut Event, peers: &mut Vec<Client>) -> ResponseCode {
    let room = match &event.kind {
        EventKind::Join(r) => r,
        _ => panic!("on_join received non-join event"),
    };

    let room = room.to_string();

    if let Some(index) = peers.iter().position(|p| p.addr.eq(&event.addr)) {
        peers[index].rooms.push(room.clone());
        server_say_to_room(&format!("{} has joined.", peers[index].user), &room, "server", peers)
    }

    ResponseCode::OK
}

fn on_leave(event: &mut Event, peers: &mut Vec<Client>) -> ResponseCode {
    let room = match &event.kind {
        EventKind::Leave(l) => l,
        _ => panic!("on_leave received non-leave event"),
    };

    let room = room.to_string();

    if let Some(index) = peers.iter().position(|p| p.addr.eq(&event.addr)) {
        if let Some(cindex) = peers[index].rooms.iter().position(|r| r.eq(&room)) {
            peers[index].rooms.remove(cindex);
            server_say_to_room(&format!("{} has left the room.", peers[index].user), &room, "server", peers);
        } 
    }

    ResponseCode::OK
}

fn on_list(event: &mut Event, peers: &mut Vec<Client>) -> ResponseCode {
    let to_list = match &event.kind {
        EventKind::List(option) => option,
        _ => panic!("on_list received non-list event"),
    };

    let mut retcode = ResponseCode::OK;

    let reply = match to_list {
        Some(room) => {
            let clients: Vec<String> = peers.iter()
                .filter(|c| c.rooms.iter().any(|r| r.eq(room)))
                .map(|c| c.user.clone())
                .collect();
            if clients.len() > 0 {
                clients.join(" ")
            } else {
                retcode = ResponseCode::ROOM_DOESNT_EXIST;
                String::from("")
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

    if let Some(index) = peers.into_iter().position(|p| p.addr.eq(&event.addr)) {
        let user = peers[index].user.clone();
        server_say_to(retcode, &reply, &user, peers);
    }

    retcode
}

fn on_say(event: &mut Event, peers: &mut Vec<Client>) -> ResponseCode {
    let (room, message) = match &event.kind {
        EventKind::Say(r, m) => (r, m),
        _ => panic!("on_say received non-say event"),
    };

    let sender = match peers.iter().position(|p| p.addr.eq(&event.addr)) {
        Some(index) => peers[index].user.clone(),
        None => String::from("unidentified"),
    };

    server_say_to_room(&message, &room, &sender, peers);

    ResponseCode::OK
}

fn on_quit(event: &mut Event, peers: &mut Vec<Client>) -> ResponseCode {

    if let Some(index) = peers.into_iter().position(|p| p.addr.eq(&event.addr)) {
        peers.remove(index);
    }

    match event.from.shutdown(net::Shutdown::Both) {
        _ => (),
    }

    ResponseCode::OK
}
