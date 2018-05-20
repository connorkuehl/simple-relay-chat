use ::net;

pub enum EventKind {
    Identify(String),
    List(Option<String>),
    Quit,
    Error,
}

pub struct Event {
    pub from: net::TcpStream,
    pub kind: EventKind,
    pub contents: String,
}

pub fn kind_parse(s: &String) -> EventKind {
    let command = match s.split_whitespace().nth(0) {
        Some(first) => first,
        None => return EventKind::Error,
    };

    let args = s.split_whitespace().skip(1).next();

    match command {
        "IDENTIFY" => identify(args),
        "LIST" => list(args),
        "QUIT" => quit(),
        _ => EventKind::Error,
    }
}

fn identify(args: Option<&str>) -> EventKind {
    match args {
        Some(username) => EventKind::Identify(username.into()),
        None => EventKind::Error,
    }
}

fn list(args: Option<&str>) -> EventKind {
    match args {
        Some(room) => EventKind::List(Some(room.into())),
        None => EventKind::List(None),
    }
}

fn quit() -> EventKind {
    EventKind::Quit
}
