use ::net;
use ::{Arc, Mutex};

#[derive(Debug)]
pub struct Client;

pub fn identify(conn: &mut net::TcpStream, clients: &Arc<Mutex<Vec<Client>>>) -> Result<Client, String> {
    Ok(Client {})
}