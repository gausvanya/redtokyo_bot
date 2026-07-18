use crate::bot::utils::text::clear_text;
use telers::types::Message;

pub fn get_user_mention(user_id: i64, username: Option<&str>, full_name: String) -> Box<str> {
    let display_name = if full_name.is_empty() {
        user_id.to_string()
    } else {
        clear_text(full_name)
    };

    let user_id_str = user_id.to_string();

    let result = if let Some(cropped_id) = user_id_str.strip_prefix("-100") {
        match username {
            Some(uname) => format!("<a href='https://t.me/{}'>{}</a>", uname, display_name),
            None => format!(
                "<a href='https://t.me/c/{}/9999999'>{}</a>",
                cropped_id, display_name
            ),
        }
    } else {
        match username {
            Some(uname) => format!("<a href='https://t.me/{}'>{}</a>", uname, display_name),
            None => format!(
                "<a href='tg://openmessage?user_id={}'>{}</a>",
                user_id, display_name
            ),
        }
    };

    result.into_boxed_str()
}

pub fn get_user_info(msg: &Message) -> (i64, Option<Box<str>>, Box<str>) {
    let (user_id, username, full_name) = if let Some(c) = msg.sender_chat() {
        (
            c.id(),
            c.username().map(|s| s.to_string().into_boxed_str()),
            c.title().unwrap_or_default().to_string().into_boxed_str(),
        )
    } else if let Some(u) = msg.from() {
        let name = format!("{} {}", u.first_name, u.last_name.as_deref().unwrap_or_default())
            .trim()
            .to_string();
        (
            u.id,
            u.username.as_deref().map(|s| s.to_string().into_boxed_str()),
            name.into_boxed_str(),
        )
    } else {
        let c = msg.chat();
        let name = format!("{} {}", c.first_name().unwrap_or_default(), c.last_name().unwrap_or_default())
            .trim()
            .to_string();
        (
            c.id(),
            c.username().map(|s| s.to_string().into_boxed_str()),
            name.into_boxed_str(),
        )
    };

    (user_id, username, full_name)
}
