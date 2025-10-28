mod logger;
mod processors;
mod settings;
use email_connect::{connect_imaps, get_unseen};
use processors::reader::read_letter;
use settings::load_env;

fn main() {
    let _guard = logger::init_logs();
    tracing::info!("app start");
    let cfg = load_env();
    let mut connection = connect_imaps(cfg.server, cfg.port, cfg.username, cfg.password);

    let ids = match get_unseen(&mut connection) {
        Some(v) if !v.is_empty() => v,
        _ => {
            println!("Нет непрочитанных писем");
            return;
        }
    };

    let mut session = connection.expect("connection failed");

    read_letter(&mut session, &ids);

    let _ = session.logout();
    tracing::warn!("done");
}
