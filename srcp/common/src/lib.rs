pub struct Message {
    pub code: usize,
    pub sender: String,
    pub time: usize,
    pub room: String,
    pub body: String,
}

impl Message {
    pub fn try_new(s: &str) -> Result<Self, ()> {
        let pieces: Vec<&str> = s.split_whitespace().collect();

        if pieces.len() < 5 {
            return Err(());
        }

        Ok( Self {
            code: pieces[0].parse::<usize>().unwrap(),
            sender: String::from(pieces[1]),
            time: pieces[2].parse::<usize>().unwrap(),
            room: String::from(pieces[3]),
            body: pieces[4..pieces.len()].join(" "),
        })
    }
}

pub enum Command {
    // IDENTIFY nickname
    Identify(String),
    // Option 1: LIST
    // Option 2: LIST room_name
    List(Option<String>),
    // JOIN room_name
    Join(String),
    // SAY room_name message goes here!
    Say(String, String),
    // WHISPER username message goes here!
    Whisper(String, String),
    // SHOUT message goes here!
    Shout(String),
    // LEAVE room_name
    Leave(String),
    // QUIT
    Quit,
    ParseError,
}

impl Command {
    pub fn new(message: &str) -> Command {
        let args: Vec<&str> = message.split_whitespace().collect();

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

