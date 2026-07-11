use crate::bot::utils::text::clear_text;
use telers::types::Message;

pub fn get_user_mention(user_id: i64, username: Option<String>, full_name: String) -> String {
    let display_name = if full_name.is_empty() {
        user_id.to_string()
    } else {
        clear_text(full_name)
    };

    let user_id_str = user_id.to_string();

    if user_id_str.strip_prefix("-100").is_some() {
        return match username {
            Some(uname) => {
                format!("<a href='https://t.me/{}'>{}</a>", uname, display_name)
            }
            None => {
                let cropped_id = &user_id_str[4..];
                format!(
                    "<a href='https://t.me/c/{}/9999999'>{}</a>",
                    cropped_id, display_name
                )
            }
        };
    }

    match username {
        Some(uname) => format!("<a href='https://t.me/{}'>{}</a>", uname, display_name),
        None => format!(
            "<a href='tg://openmessage?user_id={}'>{}</a>",
            user_id, display_name
        ),
    }
}

pub fn get_user_info(msg: &Message) -> (i64, Option<String>, String) {
    let (user_id, username, full_name) = if let Some(c) = msg.sender_chat() {
        (
            c.id(),
            c.username().map(|s| s.to_string()),
            c.title().unwrap_or_default().to_string(),
        )
    } else if let Some(u) = msg.from() {
        (
            u.id,
            u.username.as_ref().map(|s| s.to_string()),
            format!(
                "{} {}",
                u.first_name,
                u.last_name.as_deref().unwrap_or_default()
            )
            .trim()
            .to_string(),
        )
    } else {
        let c = msg.chat();
        (
            c.id(),
            c.username().map(|s| s.to_string()),
            format!(
                "{} {}",
                c.first_name().unwrap_or_default(),
                c.last_name().unwrap_or_default()
            ),
        )
    };

    (user_id, username, full_name)
}
