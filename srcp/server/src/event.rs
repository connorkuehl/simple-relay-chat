use ::net;

pub enum EventKind {
    Identify(String),
    List(Option<String>),
    Join(String),
    Leave(String),
    Say(String, String),
    Quit,
    Error,
}

pub struct Event {
    pub from: net::TcpStream,
    pub addr: net::SocketAddr,
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
        "LEAVE" => leave(args),
        "LIST" => list(args),
        "SAY" => say(args),
        "QUIT" => quit(),
        _ => EventKind::Error,
    }
}

fn identify(args: Vec<&str>) -> EventKind {
    EventKind::Identify(String::from(args[0]))
}

fn join(args: Vec<&str>) -> EventKind {
    EventKind::Join(String::from(args[0]))
}

fn leave(args: Vec<&str>) -> EventKind {
    EventKind::Leave(String::from(args[0]))
}

fn list(args: Vec<&str>) -> EventKind {
    if args.len() < 1 {
        EventKind::List(None)
    } else {
        EventKind::List(Some(String::from(args[0])))
    }
}

fn say(args: Vec<&str>) -> EventKind {
    if args.len() < 2 {
        EventKind::Error
    } else {
        let room = args[0];
        let message = args[1..args.len()].join(" ");

        EventKind::Say(String::from(room), String::from(message))
    }
}

fn quit() -> EventKind {
    EventKind::Quit
}
