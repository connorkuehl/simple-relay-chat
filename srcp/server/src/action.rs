use ::net;
use ::Client;
use ::HashMap;
use ::std::io::Write;

use ::event::{Event, EventKind};

const OK: usize = 0;

pub fn execute(event: Event, peers: &mut HashMap<net::SocketAddr, Client>) {
    let mut retcode = OK;
    let mut event = event;

    match event.kind {
        EventKind::Identify(user) => {
            if let Ok(addr) = event.from.peer_addr() {
                if !peers.contains_key(&addr) {
                    let client = Client {
                        user: user,
                        conn: event.from.try_clone().expect("try_clone"),
                    };
                    peers.insert(addr, client);
                } else {
                }
            }
        },
        _ => retcode = 999,
    }

    let response = format!("{} {}", retcode, event.contents);

    event.from.write(response.as_bytes()).expect("write");
    event.from.flush().expect("write flush");
}