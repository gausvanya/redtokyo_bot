use sea_orm::DatabaseConnection;
use telers::{Bot, Extension, types::Message};

use crate::{
    bot::{
        enums::{tg_emoji::Emoji, user_type::UserIdentity},
        filters::command::ParsedCommand,
        methods::message::MessageMethods,
        utils::user::get_user_info,
    },
    database::repo::{admins_repo::AdminRepo, user_repo::UserRepo},
};

fn parse_user_id(s: &str) -> Option<i64> {
    if let Ok(id) = s.parse::<i64>() {
        Some(id)
    } else if let Some(stripped) = s.strip_prefix('_') {
        format!("-{stripped}")
            .parse::<i64>()
            .ok()
    } else {
        None
    }
}

pub async fn db_update_command_handler(
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

    let target_id = args
        .get("user")
        .and_then(parse_user_id);

    let Some(target_id) = target_id else {
        let message_text = format!("{} Боту должен быть передан числовой ИД", Emoji::Information);

        bot.send(MessageMethods::send(&msg).text(message_text))
            .await?;
        return Ok(());
    };

    let user_repo = UserRepo::new(db);
    let user_obj = user_repo
        .get(UserIdentity::Id(target_id))
        .await?;

    if user_obj.is_some() {
        let message_text = format!("{} Данный ИД уже известен боту", Emoji::Information);

        bot.send(MessageMethods::send(&msg).text(message_text))
            .await?;
    } else {
        user_repo
            .insert(target_id, None, target_id.to_string())
            .await?;

        let message_text = format!("{} Данные пользователя обновлены в базе", Emoji::Information);

        bot.send(MessageMethods::send(&msg).text(message_text))
            .await?;
    }

    Ok(())
}
