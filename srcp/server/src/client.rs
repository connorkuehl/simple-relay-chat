use ::net;
use ::Read;
use ::{Arc, Mutex, Weak};

#[derive(Debug)]
pub struct Client {
    pub user: String,
    pub conn: Weak<Mutex<net::TcpStream>>,
}

pub fn identify(conn: &Arc<Mutex<net::TcpStream>>, clients: &Arc<Mutex<Vec<Client>>>) -> Result<Client, String> {
    let mut buffer = String::new();

    conn.lock().unwrap().read_to_string(&mut buffer).unwrap();
    let words: Vec<&str> = buffer.split(" ").collect();
    if words.len() != 2 {
        return Err("malformed".into());
    }
    let username = words[1].trim();

    let client = Client {
        user: username.into(),
        conn: Arc::downgrade(&conn),
    };

    Ok(client)
}
