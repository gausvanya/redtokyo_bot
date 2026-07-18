use crate::bot::utils::chat::is_allowed_chat;
use crate::bot::utils::user::get_user_mention;
use telers::Bot;
use telers::methods::SendMessage;
use telers::types::{ChatMember, ChatMemberUpdated};

pub async fn bot_welcome_handler(bot: Bot, event: ChatMemberUpdated) -> anyhow::Result<()> {
    let is_joined = matches!(
        event.old_chat_member,
        ChatMember::Left(_) | ChatMember::Kicked(_)
    ) && !matches!(
        event.new_chat_member,
        ChatMember::Left(_) | ChatMember::Kicked(_)
    );

    if !is_joined {
        return Ok(());
    }

    if !is_allowed_chat(&bot, event.chat.id()).await {
        return Ok(());
    }

    let user = event.new_chat_member.user();
    let user_mention = get_user_mention(
        user.id,
        user.username.as_deref(),
        user.first_name.parse()?,
    );
    let text_mention = format!(
        "<tg-emoji emoji-id='5372981976804366741'>🤖</tg-emoji> {} был добавлен в чат.\n\
            Я являюсь бото-помощником для модерации сетки чатов RedTokyo.\n\
            Назначь меня администратором чата, для начала работы",
        user_mention
    );

    bot.send(SendMessage::new(event.chat.id(), text_mention).parse_mode("HTML"))
        .await?;

    Ok(())
}
