use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::filters::get_user::GetUserInfo;
use crate::bot::keyboards::garant_call_keyboard;
use crate::bot::methods::message::MessageMethods;
use crate::bot::utils::chat::GL_ADMINS;
use crate::bot::utils::user::{get_user_info, get_user_mention};
use crate::database::cache::{SUMMON_CACHE, SummonPayload};
use crate::database::repo::garant_repo::GarantRepo;
use sea_orm::DatabaseConnection;
use telers::methods::EditMessageReplyMarkup;
use telers::types::Message;
use telers::{Bot, Extension};
use uuid::Uuid;

pub async fn set_garant_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&admin_id) {
        return Ok(());
    }

    let (user, comment) = (args.get("user"), args.require("comment").to_string());

    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let garant_repo = GarantRepo::new(db);
        let _ = garant_repo.insert(user.id, comment).await;

        let user_mention = get_user_mention(user.id, user.username, user.full_name);

        bot.send(MessageMethods::send(&msg).text(format!(
            "{} {} добавлен в список гарантов RedTokyo",
            Emoji::Balloon,
            user_mention
        )))
        .await?;
    }
    Ok(())
}

pub async fn remove_garant_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin_id = get_user_info(&msg).0;

    if !GL_ADMINS.contains(&admin_id) {
        return Ok(());
    }

    let user = args.get("user");

    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let garant_repo = GarantRepo::new(db);
        let is_garant = garant_repo.get(user.id).await?;
        let user_mention = get_user_mention(user.id, user.username, user.full_name);

        if let Some(garant_model) = is_garant {
            let _ = garant_repo.delete(garant_model).await;

            bot.send(MessageMethods::send(&msg).text(format!(
                "{} {} исключен из списка гарантов RedTokyo",
                Emoji::Balloon,
                user_mention
            )))
            .await?;
        } else {
            bot.send(MessageMethods::send(&msg).text(format!(
                "{} {} Отсутствует в списке гарантов RedTokyo",
                Emoji::Balloon,
                user_mention
            )))
            .await?;
        }
    }
    Ok(())
}

pub async fn garant_list_command_handler(
    bot: Bot,
    msg: Message,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let garant_repo = GarantRepo::new(db);
    let garants = garant_repo.get_all().await?;

    let text = if !garants.is_empty() {
        let mut buffer = String::new();

        for (garant, users) in garants {
            if let Some(user) = users.first() {
                let user_mention =
                    get_user_mention(user.id, user.username.clone(), user.full_name.clone());
                buffer.push_str(&format!(
                    "{} {} - {}\n",
                    Emoji::RadioButton,
                    user_mention,
                    garant.comment
                ));
            }
        }
        buffer
    } else {
        "Отсутствует".to_string()
    };

    let message_text = format!("{} Список гарантов чата:\n{}", Emoji::FacingUp, text);

    bot.send(MessageMethods::send(&msg).text(message_text))
        .await?;
    Ok(())
}

pub async fn garant_call_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let (user_id, username, full_name) = get_user_info(&msg);
    let user_mention = get_user_mention(user_id, username, full_name);
    let reason = args.get("reason").unwrap_or_default();

    let garant_repo = GarantRepo::new(db);
    let garants = garant_repo.get_all().await?;

    if !garants.is_empty() {
        let mut base_text = format!(
            "{} {} созывает гарантов чата",
            Emoji::Megaphone,
            user_mention
        );

        if !reason.is_empty() {
            base_text.push_str(&format!("\n\n{} {}", Emoji::Balloon, reason));
        }

        let mentions: Vec<String> = garants
            .iter()
            .map(|(garant, _)| format!(r#"<a href="tg://user?id={}">&#8296;</a>"#, garant.user_id))
            .collect();

        let chunks: Vec<Vec<String>> = mentions.chunks(5).map(|chunk| chunk.to_vec()).collect();

        let mut sent_msg_ids = Vec::new();

        for chunk in chunks {
            let text = format!("{}{}", base_text, chunk.join(""));
            let sent = bot.send(MessageMethods::send(&msg).text(text)).await?;
            sent_msg_ids.push(sent.message_id());
        }

        let summon_id = Uuid::new_v4().simple().to_string();

        let payload = SummonPayload {
            creator_id: user_id,
            msg_ids: sent_msg_ids.clone(),
        };

        SUMMON_CACHE.insert(summon_id.clone(), payload).await;

        for msg_id in sent_msg_ids {
            bot.send(
                EditMessageReplyMarkup::new()
                    .chat_id(msg.chat().id())
                    .message_id(msg_id)
                    .reply_markup(garant_call_keyboard(summon_id.clone())),
            )
            .await?;
        }
    } else {
        bot.send(MessageMethods::send(&msg).text("Список гарантов пуст."))
            .await?;
    }
    Ok(())
}
