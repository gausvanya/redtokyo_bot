use chrono::Duration;
use sea_orm::DatabaseConnection;
use telers::{Bot, Extension, types::Message};

use crate::{
    bot::{
        enums::tg_emoji::Emoji,
        filters::{command::ParsedCommand, get_user::GetUserInfo},
        methods::message::MessageMethods,
        utils::{
            datetime::get_current_datetime,
            user::{get_user_info, get_user_mention},
        },
    },
    database::repo::{admins_repo::AdminRepo, verbal_warns_repo::VerbalWarnsRepo},
};

pub async fn set_warn_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin_id = get_user_info(&msg).0;

    let admin_repo = AdminRepo::new(db.clone());

    if !admin_repo
        .get(admin_id)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let (user, reason) = (
        args.get("user"),
        args.require("reason")
            .to_string(),
    );

    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        let msg_send = bot
            .send(MessageMethods::send(&msg).text(format!(
                "{} Пользователю {} выдано устное предупреждение\n{} Причина: {}",
                Emoji::Warning,
                user_mention,
                Emoji::Balloon,
                reason
            )))
            .await?;

        let warn_repo = VerbalWarnsRepo::new(db);
        let timestamp = get_current_datetime() + Duration::days(1);

        warn_repo
            .insert(msg.chat().id(), user.id, admin_id, msg_send.message_id(), reason, timestamp)
            .await?;
    }
    Ok(())
}

pub async fn remove_warn_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin_id = get_user_info(&msg).0;

    let admin_repo = AdminRepo::new(db.clone());

    if !admin_repo
        .get(admin_id)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let user = args.get("user");

    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);
        let warn_repo = VerbalWarnsRepo::new(db);

        let user_warn = warn_repo
            .get(msg.chat().id(), user.id)
            .await?;

        if let Some(user_warn) = user_warn {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Пользователю {} снято последнее устное предупреждение",
                Emoji::Warning,
                user_mention,
            )))
            .await?;

            warn_repo
                .delete(user_warn)
                .await?;
        } else {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} У пользователя {} отсутствуют устные предупреждения",
                Emoji::Warning,
                user_mention,
            )))
            .await?;
        }
    }
    Ok(())
}

pub async fn list_warns_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let command = args.require("command");
    let normalized_command = command.to_lowercase();
    let normalized_command = normalized_command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let (user_id, username, full_name) = if normalized_command == "мои усты" {
        let user_obj = get_user_info(&msg);
        (user_obj.0, user_obj.1, user_obj.2)
    } else {
        let user = args.get("user");

        let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
            .resolve(&msg)
            .await?;

        if let Some(user) = user_obj {
            (
                user.id,
                user.username
                    .map(|s| s.into_boxed_str()),
                user.full_name
                    .into_boxed_str(),
            )
        } else {
            return Ok(());
        }
    };

    let user_mention = get_user_mention(user_id, username.as_deref(), full_name.to_string());

    let mut message_text = format!("{} Устные предупреждения {}:\n", Emoji::Warning, user_mention);

    let warn_repo = VerbalWarnsRepo::new(db);

    let warns = warn_repo
        .get_all_from_user(msg.chat().id(), user_id)
        .await?;

    if warns.is_empty() {
        message_text += "Отсутствуют."
    } else {
        for (idx, warn) in warns.iter().enumerate() {
            let time_left = warn.timestamp - get_current_datetime();

            let expires_in = if time_left.num_seconds() <= 0 {
                "снимается...".to_string()
            } else {
                format!("{} ч. {} мин.", time_left.num_hours(), time_left.num_minutes() % 60)
            };

            let url = format!(
                "https://t.me/c/{}/{}",
                warn.chat_id
                    .to_string()
                    .replace("-100", ""),
                warn.message_id
            );
            message_text.push_str(&format!(
                "От [<a href='{}'>{}</a>]: {}\nИстекает: {}\n\n",
                url,
                idx + 1,
                warn.reason,
                expires_in
            ));
        }
    }

    bot.send(MessageMethods::send(&msg).text(message_text))
        .await?;
    Ok(())
}
