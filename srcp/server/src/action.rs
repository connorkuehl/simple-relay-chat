use ::Client;
use ::std::io::Write;

use ::event::{Event, EventKind};

const OK: usize = 0;
const USERNAME_UNAVAILABLE: usize = 4;

pub fn execute(event: Event, peers: &mut Vec<Client>) {
    let mut event = event;

    let retcode = match event.kind {
        EventKind::Identify(_) => on_identify(&mut event, peers),
        _ => 0,
    };

    let response = format!("{} {}", retcode, event.contents);
    event.from.write(response.as_bytes()).expect("write");
    event.from.flush().expect("write flush");
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
