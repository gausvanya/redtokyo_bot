use sea_orm::DatabaseConnection;
use telers::{
    Bot, Extension,
    methods::SendMessage,
    types::{ChatMember, ChatMemberUpdated},
};

use crate::{
    bot::utils::{chat::is_allowed_chat, user::UserMention},
    database::repo::user_repo::UserRepo,
};

pub async fn bot_welcome_handler(
    bot: Bot,
    event: ChatMemberUpdated,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let is_joined = matches!(event.old_chat_member, ChatMember::Left(_) | ChatMember::Kicked(_))
        && !matches!(event.new_chat_member, ChatMember::Left(_) | ChatMember::Kicked(_));

    if !is_joined {
        return Ok(());
    }

    let user_repo = UserRepo::new(db.clone());
    let chat = event.chat;

    user_repo
        .insert(
            chat.id(),
            chat.username()
                .map(|s| s.to_string()),
            chat.title()
                .unwrap_or_default()
                .to_string(),
        )
        .await?;

    if !is_allowed_chat(&bot, chat.id(), db).await? {
        return Ok(());
    }

    let text_mention = format!(
        "<tg-emoji emoji-id='5372981976804366741'>🤖</tg-emoji> {} был добавлен в чат.\nЯ являюсь \
         бото-помощником для модерации сетки чатов RedTokyo.\nНазначь меня администратором чата, \
         для начала работы",
        event
            .new_chat_member
            .user()
            .mention()
    );

    bot.send(SendMessage::new(chat.id(), text_mention).parse_mode("HTML"))
        .await?;

    Ok(())
}
