use ::client::Client;

pub enum Event {
    Identify(Client),
    Unknown,
}

pub fn parse(cmdstr: String) -> Result<Event, String> {
    Ok(Event::Unknown)
}