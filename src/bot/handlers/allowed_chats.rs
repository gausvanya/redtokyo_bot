use sea_orm::DatabaseConnection;
use telers::{Bot, Extension, methods::LeaveChat, types::Message};

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
    database::repo::allowed_chats_repo::AllowedChatsRepo,
};

pub async fn allowed_chats_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&admin_id) {
        return Ok(());
    }

    let allowed_chat_id = args.require("user");
    let chat_obj = GetUserInfo::new(
        allowed_chat_id
            .to_string()
            .into(),
        &db,
        bot.clone(),
    )
    .resolve(&msg)
    .await?;

    if let Some(chat) = chat_obj {
        let allowed_chat_repo = AllowedChatsRepo::new(db.clone());
        let chat_result = allowed_chat_repo
            .get(chat.id)
            .await?;
        let chat_mention = get_user_mention(chat.id, chat.username.as_deref(), chat.full_name);

        if chat_result.is_some() {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Чат {} уже был добавлен в список разрешенных",
                Emoji::Balloon,
                chat_mention
            )))
            .await?;
        } else {
            allowed_chat_repo
                .insert(chat.id)
                .await?;

            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Чат {} был добавлен в список разрешенных",
                Emoji::Balloon,
                chat_mention
            )))
            .await?;
        }
    }
    Ok(())
}

pub async fn rem_allowed_chats_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&admin_id) {
        return Ok(());
    }

    let allowed_chat_id = args.require("user");
    let chat_obj = GetUserInfo::new(
        allowed_chat_id
            .to_string()
            .into(),
        &db,
        bot.clone(),
    )
    .resolve(&msg)
    .await?;

    if let Some(chat) = chat_obj {
        let allowed_chat_repo = AllowedChatsRepo::new(db.clone());
        let chat_result = allowed_chat_repo
            .get(chat.id)
            .await?;
        let chat_mention = get_user_mention(chat.id, chat.username.as_deref(), chat.full_name);

        if let Some(chat_res) = chat_result {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Чат {} был удален из списка разрешенных",
                Emoji::Balloon,
                chat_mention
            )))
            .await?;

            let _ = bot
                .send(LeaveChat::new(chat_res.chat_id))
                .await;

            allowed_chat_repo
                .delete(chat_res)
                .await?;
        } else {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} Чат {} не состоит в списке разрешенных",
                Emoji::Balloon,
                chat_mention
            )))
            .await?;
        }
    }
    Ok(())
}
