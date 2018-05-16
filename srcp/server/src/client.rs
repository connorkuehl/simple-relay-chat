use ::net;

pub struct Client;

pub fn identify(conn: &mut net::TcpStream) -> Result<Client, String> {
    Ok(Client {})
}