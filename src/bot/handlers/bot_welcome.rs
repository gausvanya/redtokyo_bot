use telers::{
    Bot,
    methods::SendMessage,
    types::{ChatMember, ChatMemberUpdated},
};

use crate::bot::utils::{chat::is_allowed_chat, user::UserMention};

pub async fn bot_welcome_handler(bot: Bot, event: ChatMemberUpdated) -> anyhow::Result<()> {
    let is_joined = matches!(event.old_chat_member, ChatMember::Left(_) | ChatMember::Kicked(_))
        && !matches!(event.new_chat_member, ChatMember::Left(_) | ChatMember::Kicked(_));

    if !is_joined {
        return Ok(());
    }

    if !is_allowed_chat(&bot, event.chat.id()).await {
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

    bot.send(SendMessage::new(event.chat.id(), text_mention).parse_mode("HTML"))
        .await?;

    Ok(())
}
