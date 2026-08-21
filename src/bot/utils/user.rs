use std::borrow::Cow;

use telers::types::{Message, User};

use crate::bot::utils::text::clear_text;

pub fn get_user_mention(user_id: i64, username: Option<&str>, full_name: String) -> Box<str> {
    let full_name = clear_text(full_name);

    let display_name: Cow<str> = if !full_name.is_empty() {
        Cow::Owned(full_name)
    } else if let Some(uname) = username {
        Cow::Borrowed(uname)
    } else {
        Cow::Owned(user_id.to_string())
    };

    let href = match username {
        Some(uname) => format!("https://t.me/{uname}"),
        None => {
            const SUPERGROUP_PREFIX: i64 = -1_000_000_000_000;
            if user_id <= SUPERGROUP_PREFIX {
                let chat_id = user_id - SUPERGROUP_PREFIX;
                format!("tg://openmessage?chat_id={chat_id}")
            } else {
                format!("tg://openmessage?user_id={user_id}")
            }
        }
    };

    format!("<a href='{href}'>{display_name}</a>").into_boxed_str()
}

pub trait UserMention {
    fn mention(&self) -> Box<str>;
}

impl UserMention for User {
    fn mention(&self) -> Box<str> {
        get_user_mention(
            self.id,
            self.username.as_deref(),
            self.first_name
                .to_string(),
        )
    }
}

pub fn get_user_info(msg: &Message) -> (i64, Option<Box<str>>, Box<str>) {
    let (user_id, username, full_name) = if let Some(c) = msg.sender_chat() {
        (
            c.id(),
            c.username().map(|s| {
                s.to_string()
                    .into_boxed_str()
            }),
            c.title()
                .unwrap_or_default()
                .to_string()
                .into_boxed_str(),
        )
    } else if let Some(u) = msg.from() {
        let name = format!(
            "{} {}",
            u.first_name,
            u.last_name
                .as_deref()
                .unwrap_or_default()
        )
        .trim()
        .to_string();
        (
            u.id,
            u.username
                .as_deref()
                .map(|s| {
                    s.to_string()
                        .into_boxed_str()
                }),
            name.into_boxed_str(),
        )
    } else {
        let c = msg.chat();
        let name = format!(
            "{} {}",
            c.first_name()
                .unwrap_or_default(),
            c.last_name()
                .unwrap_or_default()
        )
        .trim()
        .to_string();
        (
            c.id(),
            c.username().map(|s| {
                s.to_string()
                    .into_boxed_str()
            }),
            name.into_boxed_str(),
        )
    };

    (user_id, username, full_name)
}
