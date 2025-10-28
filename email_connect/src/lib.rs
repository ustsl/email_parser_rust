use imap::Session;
use native_tls::TlsConnector;
use std::net::TcpStream;

pub struct EmailConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub type ImapSession = Session<native_tls::TlsStream<TcpStream>>;

fn load_email_config(server: String, port: u16, username: String, password: String) -> EmailConfig {
    EmailConfig {
        server,
        port,
        username,
        password,
    }
}

pub fn connect_imaps(
    server: String,
    port: u16,
    username: String,
    password: String,
) -> imap::error::Result<ImapSession> {
    let cfg = load_email_config(server, port, username, password);
    println!("{}, {}", cfg.server, cfg.username);
    let tls = TlsConnector::builder().build()?;
    let client = imap::connect((cfg.server.as_str(), cfg.port), cfg.server.as_str(), &tls)?;
    let session = client
        .login(&cfg.username, &cfg.password)
        .map_err(|e| e.0)?;
    Ok(session)
}

pub fn get_unseen(connection: &mut imap::error::Result<ImapSession>) -> Option<Vec<u32>> {
    if let Ok(sess) = connection.as_mut() {
        sess.select("INBOX").ok()?;

        let ids: Vec<u32> = sess.search("UNSEEN").ok()?.into_iter().collect();

        if ids.is_empty() {
            println!("Нет непрочитанных писем");
            return None;
        }

        println!("{} непрочитанных писем", ids.len());
        Some(ids)
    } else {
        eprintln!("connection failed");
        None
    }
}
