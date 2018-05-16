use ::client::Client;

pub enum Event {
    Identify(Client),
    Unknown(String),
}

pub fn parse(cmdstr: &String) -> Event {
    Event::Unknown(cmdstr.clone())
}
