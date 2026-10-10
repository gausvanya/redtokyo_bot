use sea_orm::{DatabaseConnection, IntoOption};
use telers::{
    Bot, Extension,
    methods::{BanChatMember, UnbanChatMember},
    types::Message,
};

use crate::{
    bot::{
        enums::tg_emoji::Emoji,
        filters::{command::ParsedCommand, get_user::GetUserInfo},
        methods::message::MessageMethods,
        utils::{
            chat::OWNER_IDS,
            user::{get_user_info, get_user_mention},
        },
    },
    database::repo::{allowed_chats_repo::AllowedChatsRepo, bans_repo::BansRepo},
};

pub async fn ban_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let sender_id = get_user_info(&msg).0;

    if !OWNER_IDS.contains(&sender_id) {
        return Ok(());
    }

    let (user_id, reason) = (args.get("user"), args.get("reason"));
    let user_obj = GetUserInfo::new(user_id.into_option(), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let ban_repo = BansRepo::new(db.clone());
        let allowed_chat_repo = AllowedChatsRepo::new(db);
        let allowed_chat_ids = allowed_chat_repo
            .get_all()
            .await?;

        let max_chats = allowed_chat_ids.len();
        let mut success_count: u8 = 0;

        for chat in allowed_chat_ids.iter() {
            if bot
                .send(BanChatMember::new(chat.chat_id, user.id))
                .await
                .is_ok()
            {
                success_count += 1
            }
        }

        ban_repo
            .insert(user.id, sender_id, reason.into_option())
            .await?;

        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        bot.send(MessageMethods::send(&msg).text(format!(
            "{} {} исключен из {} / {} чатов\n{} Вход в чаты сетки чатов RedTokyo закрыт",
            Emoji::Bangbang,
            user_mention,
            success_count,
            max_chats,
            Emoji::Balloon
        )))
        .await?;
    }
    Ok(())
}

pub async fn unban_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let sender_id = get_user_info(&msg).0;

    if !OWNER_IDS.contains(&sender_id) {
        return Ok(());
    }

    let user_id = args.get("user");
    let user_obj = GetUserInfo::new(user_id.into_option(), &db, bot.clone())
        .resolve(&msg)
        .await?;

    let Some(user) = user_obj else {
        return Ok(());
    };

    let ban_repo = BansRepo::new(db.clone());
    let allowed_chat_repo = AllowedChatsRepo::new(db);
    let allowed_chat_ids = allowed_chat_repo
        .get_all()
        .await?;

    let max_chats = allowed_chat_ids.len();
    let mut success_count: u8 = 0;

    let ban = ban_repo
        .get(user.id)
        .await?;

    if let Some(ban) = ban {
        for chat in allowed_chat_ids.iter() {
            if bot
                .send(UnbanChatMember::new(chat.chat_id, user.id).only_if_banned(true))
                .await
                .is_ok()
            {
                success_count += 1;
            }
        }
        ban_repo
            .delete(ban)
            .await?;

        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        bot.send(MessageMethods::send(&msg).text(format!(
            "{} {} разбанен в {} / {} чатах\n{} Вход в чаты сетки RedTokyo снова открыт",
            Emoji::Bangbang,
            user_mention,
            success_count,
            max_chats,
            Emoji::Balloon
        )))
        .await?;
    } else {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        bot.send(MessageMethods::send(&msg).text(format!(
            "{} {} не забанен в сетке чатов RedTokyo",
            Emoji::Bangbang,
            user_mention,
        )))
        .await?;
    }

    Ok(())
}
