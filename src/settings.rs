use dotenvy::dotenv;
use std::env;

pub struct EnvConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub fn load_env() -> EnvConfig {
    dotenv().ok();

    let server = env::var("APPOINTMENT_IMAP_SERVER").expect("server not set");
    let port = env::var("APPOINTMENT_IMAP_PORT")
        .expect("port not set")
        .parse::<u16>()
        .expect("port must be number");
    let username = env::var("APPOINTMENT_USERNAME").expect("username not set");
    let password = env::var("APPOINTMENT_PASSWORD").expect("pw not set");

    EnvConfig {
        server,
        port,
        username,
        password,
    }
}
