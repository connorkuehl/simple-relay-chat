use ::net;

pub enum EventKind {
    Identify(String),
    Error(String),
}

pub struct Event {
    pub from: net::TcpStream,
    pub kind: EventKind,
    pub contents: String,
}

pub fn kind_parse(s: &String) -> EventKind {
    let command = match s.split_whitespace().nth(0) {
        Some(first) => first,
        None => return EventKind::Error("no command".into()),
    };

    let args = s.split_whitespace().skip(1).next();

    match command {
        "IDENTIFY" => identify(args.unwrap()),
        _ => EventKind::Error("command not recognized".into()),
    }
}

fn identify(args: &str) -> EventKind {
    EventKind::Identify(args.into())    
}
