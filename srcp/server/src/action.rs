use ::net;
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

    let retcode = match event.kind {
        EventKind::Identify(_) => on_identify(&mut event, peers),
        EventKind::Quit => on_quit(&mut event, peers),
        _ => 0,
    };

    let response = format!("{} {}", retcode, event.contents);
    match event.from.write(response.as_bytes()) {
        _ => (),
    }
    match event.from.flush() {
        _ => (),
    }
}

fn on_identify(event: &mut Event, peers: &mut Vec<Client>) -> usize {
    let username = match &event.kind {
        EventKind::Identify(user) => user,
        _ => panic!("on_identify received non-identify event"),
    };

    if peers.iter().any(|p| p.user.eq(username)) {
        return USERNAME_UNAVAILABLE;
    }

    let client = Client {
        user: username.to_string(),
        conn: event.from.try_clone().expect("try_clone on_identify"),
    };

    peers.push(client);

    OK
}

fn on_quit(event: &mut Event, peers: &mut Vec<Client>) -> usize {
    // send message to subscribed channels saying they left
    // probably just call on_leave for each of them.

    let addr = event.from.peer_addr().expect("peer_addr");
    if let Some(index) = peers.iter().position(|x| x.conn.peer_addr().expect("peer_addr").eq(&addr)) {
        peers.remove(index);
    }

    match event.from.shutdown(net::Shutdown::Both) {
        _ => (),
    }

    0
}
