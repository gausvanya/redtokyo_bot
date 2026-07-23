use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::keyboards::{duel_chat_keyboard, duel_unmute_keyboard};
use crate::bot::methods::message::MessageMethods;
use crate::bot::utils::chat::{ADMIN_CHAT_ID, DUEL_CHAT_ID};
use crate::bot::utils::parse::parse_amount;
use crate::bot::utils::user::{get_user_info, get_user_mention};
use crate::database::cache::WARN_CACHE;
use telers::methods::RestrictChatMember;
use telers::types::{ChatPermissions, Message, ReplyParameters};
use telers::{Bot, Extension};
use crate::bot::utils::trade::get_minimum_duel_rate;

pub async fn duel_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    if msg.chat().id() != DUEL_CHAT_ID {
        return Ok(());
    }

    let chat_id = msg.chat().id();
    let (user_id, username, full_name) = get_user_info(&msg);

    let raw_amount = args.get("amount").unwrap_or("0");
    let amount = parse_amount(raw_amount);

    if amount == -1 {
        return Ok(());
    }

    let duel_rate = get_minimum_duel_rate().await?;
    let min_bet = duel_rate.min_bet as i64;

    if (0..min_bet).contains(&amount) {
        let key = format!("warns_low_bet:{chat_id}:{user_id}");

        let warns = WARN_CACHE.get(&key).await.unwrap_or(0) + 1;
        WARN_CACHE.insert(key.clone(), warns).await;

        if warns >= 2 {
            WARN_CACHE.remove(key.as_str()).await;
            let until_date = (chrono::Utc::now() + chrono::Duration::minutes(5)).timestamp();

            let permissions = ChatPermissions {
                can_send_messages: Some(false),
                can_send_audios: Some(false),
                can_send_documents: Some(false),
                can_send_photos: Some(false),
                can_send_videos: Some(false),
                can_send_video_notes: Some(false),
                can_send_polls: Some(false),
                can_send_other_messages: Some(false),
                can_add_web_page_previews: Some(false),
                can_react_to_messages: Some(false),
                can_edit_tag: Some(false),
                can_change_info: Some(false),
                can_invite_users: Some(false),
                can_pin_messages: Some(false),
                can_send_voice_notes: Some(false),
                can_manage_topics: Some(false),
            };

            if bot
                .send(RestrictChatMember::new(chat_id, user_id, permissions).until_date(until_date))
                .await
                .is_err()
            {
                return Ok(());
            }

            let user_mention = get_user_mention(user_id, username.as_deref(), full_name.to_string());

            let msg_sent = bot.send(
                MessageMethods::send(&msg).text(
                    format!("🤐 {user_mention}\nВы получили мут на 5 минут за повторную попытку игры на сумму менее {min_bet} голд.")
                ).reply_markup(duel_unmute_keyboard(msg.chat().id(), user_id, min_bet, None))
            ).await?;

            bot.send(
                MessageMethods::send(&msg)
                    .chat_id(ADMIN_CHAT_ID)
                    .text(format!(
                        "ℹ️ {user_mention} лишен права слова в чате\n\
                    Причина: неверная ставка в играх"
                    ))
                    .reply_markup(duel_unmute_keyboard(
                        msg.chat().id(),
                        user_id,
                        Some(msg_sent.message_id()),
                    ))
                    .reply_parameters(ReplyParameters::new().chat_id(chat_id).message_id(msg_sent.message_id())),
            )
            .await?;

            WARN_CACHE.invalidate(&key).await;
        } else {
            let message_text = format!(
                "{} внимание!\n\n\
                В нашем чате разрешены игры только от 50 голд, другие варианты игр будут наказываться при повторной попытке сыграть.\n\n\
                {} Подробнее в <code>Заметка 2</code> и <code>Заметка 3</code>\n\n\
                Игры на меньшие суммы по кнопке ниже {}",
                Emoji::Bangbang,
                Emoji::Balloon,
                Emoji::ArrowDown
            );

            bot.send(
                MessageMethods::send(&msg)
                    .text(message_text)
                    .reply_markup(duel_chat_keyboard()),
            )
            .await?;
        }
    }
    Ok(())
}


pub async fn minimal_rate_duel_command_handler(
    bot: Bot,
    msg: Message,
) -> anyhow::Result<()> {
    if msg.chat().id() != DUEL_CHAT_ID {
        return Ok(());
    }

    let duel_rate = get_minimum_duel_rate().await?;
    let message_text = format!(
        "{} Минимальная ставка игр: {} ирис-голд.\n{} Курс биржи: {:.2}",
        Emoji::Gold, duel_rate.min_bet, Emoji::Trade, duel_rate.rate
    );

    bot.send(MessageMethods::send(&msg).text(message_text)).await?;
    Ok(())
}