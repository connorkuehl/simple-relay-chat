#[derive(Debug)]
pub enum Command {
    Identify(String),
    List(Option<String>),
    Join(String),
    Say(String, String),
    Whisper(String, String),
    Shout(String),
    Leave(String),
    Quit,
    ParseError,
}

impl Command {
    pub fn new(args: &str) -> Command {
        let args: Vec<&str> = args.split_whitespace().collect();

        match args[0] {
            "IDENTIFY" => Command::Identify(args[1].to_string()),
            "LIST" => {
                if args.len() > 1 {
                    Command::List(Some(args[1].to_string()))
                } else {
                    Command::List(None)
                }
            },
            "JOIN" => Command::Join(args[1].to_string()),
            "SAY" => {
                if args.len() > 2 {
                    Command::Say(args[1].to_string(), args[2..args.len()].join(" "))
                } else {
                    Command::ParseError
                }
            },
            "WHISPER" => {
                if args.len() > 2 {
                    Command::Whisper(args[1].to_string(), args[2..args.len()].join(" "))
                } else {
                    Command::ParseError
                }
            },
            "SHOUT" => {
                if args.len() >= 2 {
                    Command::Shout(args[1..args.len()].join(" "))
                } else {
                    Command::ParseError
                }
            },
            "LEAVE" => {
                if args.len() == 2 {
                    Command::Leave(args[1].to_string())
                } else {
                    Command::ParseError
                }
            },
            "QUIT" => Command::Quit,
            _ => Command::ParseError,
        }

    }
}
