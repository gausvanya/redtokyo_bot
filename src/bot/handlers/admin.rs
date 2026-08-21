use sea_orm::{DatabaseConnection, IntoOption};
use telers::{Bot, Extension, types::Message};

use crate::{
    bot::{
        enums::tg_emoji::Emoji,
        filters::{command::ParsedCommand, get_user::GetUserInfo},
        methods::message::MessageMethods,
        utils::{
            chat::GL_ADMINS,
            user::{get_user_info, get_user_mention},
        },
    },
    database::repo::admins_repo::AdminRepo,
};

pub async fn add_admin_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let sender_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&sender_id) {
        return Ok(());
    }

    let admin_id = args.get("user");
    let admin_obj = GetUserInfo::new(admin_id.into_option(), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(chat) = admin_obj {
        let admin_repo = AdminRepo::new(db.clone());
        let admin_result = admin_repo
            .get(chat.id)
            .await?;
        let user_mention = get_user_mention(chat.id, chat.username.as_deref(), chat.full_name);

        if admin_result.is_some() {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Пользователь {} уже был добавлен в список администраторов",
                Emoji::Human,
                user_mention
            )))
            .await?;
        } else {
            admin_repo
                .insert(chat.id)
                .await?;

            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Пользователь {} был добавлен в список администраторов",
                Emoji::Human,
                user_mention
            )))
            .await?;
        }
    }
    Ok(())
}

pub async fn rem_admin_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let sender_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&sender_id) {
        return Ok(());
    }

    let admin_id = args.get("user");
    let admin_obj = GetUserInfo::new(admin_id.into_option(), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(chat) = admin_obj {
        let admin_repo = AdminRepo::new(db.clone());
        let admin_result = admin_repo
            .get(chat.id)
            .await?;
        let user_mention = get_user_mention(chat.id, chat.username.as_deref(), chat.full_name);

        if let Some(admin_res) = admin_result {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Пользователь {} был удален из списка администраторов",
                Emoji::Human,
                user_mention
            )))
            .await?;

            admin_repo
                .delete(admin_res)
                .await?;
        } else {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Пользователь {} отсуствует в списке администраторов",
                Emoji::Human,
                user_mention
            )))
            .await?;
        }
    }
    Ok(())
}

pub async fn list_admin_command_handler(
    bot: Bot,
    msg: Message,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let sender_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&sender_id) {
        return Ok(());
    }

    let admin_repo = AdminRepo::new(db.clone());
    let admins = admin_repo
        .get_all()
        .await?;
    let mut buffer = String::new();

    let text = if !admins.is_empty() {
        for (_, users) in admins {
            if let Some(user) = users.first() {
                let user_mention =
                    get_user_mention(user.id, user.username.as_deref(), user.full_name.clone());
                buffer.push_str(&format!("{} {}\n", Emoji::RadioButton, user_mention,));
            }
        }
        buffer
    } else {
        "Пуст".to_string()
    };

    let message_text = format!("{} Список администраторов:\n{}", Emoji::FacingUp, text);

    bot.send(MessageMethods::send(&msg).text(message_text))
        .await?;
    Ok(())
}
