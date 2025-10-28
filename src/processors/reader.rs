use email_connect::ImapSession;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Rule {
    sender: String,
    header: String,
}
type Rules = HashMap<String, Rule>;

fn to_lower(s: &str) -> String {
    s.chars().flat_map(|c| c.to_lowercase()).collect()
}

fn extract_email(from_line: &str) -> Option<String> {
    let s = from_line.trim();
    if let (Some(l), Some(r)) = (s.find('<'), s.find('>')) {
        if r > l + 1 {
            return Some(to_lower(&s[l + 1..r]));
        }
    }
    s.split_whitespace()
        .find(|t| t.contains('@'))
        .map(|t| to_lower(t.trim_matches(|c: char| ",;\"'()<>".contains(c))))
}

fn sender_matches(rule_sender: &str, email: &str) -> bool {
    let rule_sender = rule_sender.trim();
    if rule_sender.is_empty() {
        return true;
    }
    let rs = to_lower(rule_sender);
    let em = to_lower(email.trim());
    if let Some(dom) = rs.strip_prefix('@') {
        if let Some(at) = em.rfind('@') {
            return &em[at + 1..] == dom;
        }
        false
    } else {
        em == rs
    }
}

fn header_matches(rule_header: &str, subject: &str) -> bool {
    let rh_raw = rule_header.trim();
    if rh_raw.is_empty() {
        return true;
    }
    let rh = to_lower(rh_raw);
    let sub = to_lower(subject.trim());
    if rh.contains('%') {
        let needle: String = rh.chars().filter(|&c| c != '%').collect();
        if needle.is_empty() {
            return true;
        }
        sub.contains(&needle)
    } else {
        sub == rh
    }
}

fn load_rules() -> Rules {
    let path = std::path::Path::new("rules.json");
    let data = std::fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("rules.json не найден в корне проекта");
        "{}".to_string()
    });

    let mut rules: Rules = serde_json::from_str(&data).unwrap_or_default();

    for r in rules.values_mut() {
        r.sender = r.sender.trim().to_string();
        r.header = r.header.trim().to_string();
    }

    eprintln!(
        "rules path: {}",
        std::fs::canonicalize(path).unwrap().display()
    );
    rules
}

fn parse_from_and_subject(raw: &str) -> (String, String) {
    let mut from = String::new();
    let mut subject = String::new();
    for line in raw.lines() {
        let l = line.trim();
        let ll = l.to_ascii_lowercase();
        if from.is_empty() && ll.starts_with("from:") {
            from = l.to_string();
        } else if subject.is_empty() && ll.starts_with("subject:") {
            subject = l.strip_prefix("Subject:").unwrap_or(l).trim().to_string();
        }
        if !from.is_empty() && !subject.is_empty() {
            break;
        }
    }
    (from, subject)
}

pub fn read_letter(session: &mut ImapSession, ids: &[u32]) {
    let rules = load_rules();

    for id in ids {
        let fetches = session
            .fetch(
                id.to_string(),
                "(UID BODY.PEEK[] BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])",
            )
            .expect("fetch failed");

        if let Some(msg) = fetches.iter().next() {
            let uid_opt = msg.uid; // Option<u32>

            let body = match msg.body() {
                Some(b) => b,
                None => continue,
            };
            let raw = String::from_utf8_lossy(body);

            // разбор From/Subject
            let (from_line, subject) = parse_from_and_subject(&raw);
            let email = extract_email(&from_line).unwrap_or_default();

            // матч правил
            let matched = rules.iter().find(|(_, r)| {
                sender_matches(&r.sender, &email) && header_matches(&r.header, &subject)
            });

            // вывод
            println!("\n--- Непрочитанное письмо ---");
            println!("SEQ ID: {}", id);
            match uid_opt {
                Some(u) => println!("UID: {}", u),
                None => println!("UID: недоступен"),
            }
            if subject.is_empty() {
                println!("(без темы)");
            } else {
                println!("Subject: {}", subject);
            }
            println!("EMAIL: {}", email);

            match matched {
                Some((name, _)) => println!("соответствует {}", name),
                None => println!("не соответствует"),
            }
        }
    }
}
