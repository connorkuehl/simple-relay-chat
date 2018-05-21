use ::net;

pub enum EventKind {
    Identify(String),
    List(Option<String>),
    Join(String),
    Say(String, String),
    Quit,
    Error,
}

pub struct Event {
    pub from: net::TcpStream,
    pub kind: EventKind,
    pub contents: String,
}

pub fn kind_parse(s: &str) -> EventKind {
    let to_parse = s.trim();

    let command = match to_parse.split_whitespace().nth(0) {
        Some(first) => first,
        None => return EventKind::Error,
    };

    let args: Vec<&str> = to_parse.split_whitespace().skip(1).collect();

    match command {
        "IDENTIFY" => identify(args),
        "JOIN" => join(args),
        "LIST" => list(args),
        "SAY" => say(args),
        "QUIT" => quit(),
        _ => EventKind::Error,
    }
}

fn identify(args: Vec<&str>) -> EventKind {
    EventKind::Identify(args[0].into())
}

fn join(args: Vec<&str>) -> EventKind {
    EventKind::Join(args[0].into())
}

fn list(args: Vec<&str>) -> EventKind {
    if args.len() < 1 {
        EventKind::List(None)
    } else {
        EventKind::List(Some(args[0].into()))
    }
}

fn say(args: Vec<&str>) -> EventKind {
    if args.len() < 2 {
        EventKind::Error
    } else {
        let room = args[0];
        let message = args[1..args.len()].join(" ");

        EventKind::Say(room.into(), message.into())
    }
}

fn quit() -> EventKind {
    EventKind::Quit
}
