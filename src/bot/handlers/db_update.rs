use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::methods::message::MessageMethods;
use crate::bot::utils::chat::GL_ADMINS;
use crate::bot::utils::user::get_user_info;
use crate::database::repo::user_repo::UserRepo;
use sea_orm::DatabaseConnection;
use telers::types::Message;
use telers::{Bot, Extension};
use crate::bot::enums::user_type::UserIdentity;

fn parse_user_id(s: &str) -> Option<i64> {
    if let Ok(id) = s.parse::<i64>() {
        Some(id)
    } else if let Some(stripped) = s.strip_prefix('_') {
        format!("-{stripped}").parse::<i64>().ok()
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

    if !GL_ADMINS.contains(&admin_id) {
        return Ok(());
    }

    let target_id = args.get("user").and_then(parse_user_id);

    let Some(target_id) = target_id else {
        let message_text = format!(
            "{} Боту должен быть передан числовой ИД",
            Emoji::Information
        );

        bot.send(MessageMethods::send(&msg).text(message_text))
            .await?;
        return Ok(());
    };

    let user_repo = UserRepo::new(db);
    let user_obj = user_repo.get(UserIdentity::Id(target_id)).await?;

    if let Some(_) = user_obj {
        let message_text = format!("{} Данный ИД уже известен боту", Emoji::Information);

        bot.send(MessageMethods::send(&msg).text(message_text))
            .await?;
    } else {
        user_repo.insert(target_id, None, target_id.to_string()).await?;
    }

    Ok(())
}
