use crate::bot::utils::datetime::get_current_datetime;
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Moscow;
use sea_orm::DatabaseConnection;

use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::libs::iris_api::IrisAPI;
use crate::bot::utils::chat::ADMIN_IDS;
use crate::bot::utils::user::get_user_mention;
use crate::config::get_config;
use crate::database::cache::SUMMON_CACHE;
use crate::database::repo::captcha_repo::CaptchaRepo;
use crate::database::repo::garant_repo::GarantRepo;
use telers::event::simple::HandlerResult;
use telers::methods::{
    AnswerCallbackQuery, ApproveChatJoinRequest, BanChatMember, DeclineChatJoinRequest,
    DeleteMessages, EditMessageReplyMarkup, EditMessageText, GetChatMember, RestrictChatMember,
    SendMessage,
};
use telers::types::{
    CallbackQuery, ChatMember, ChatPermissions, LinkPreviewOptions, ReplyParameters,
};
use telers::{Bot, Extension};

pub async fn captcha_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
    let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
    let code = args.require("code");

    let message_id = match call.message {
        Some(m) => m.message_id(),
        None => return Ok(()),
    };

    if code == "3" {
        let captcha_repo = CaptchaRepo::new(db.clone());
        let _ = captcha_repo.insert(chat_id, user_id).await;

        bot.send(
            EditMessageText::new()
                .chat_id(user_id)
                .text("✅ Заявка в чат принята!")
                .message_id(message_id),
        )
        .await?;

        bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
            .await?;
    } else {
        bot.send(
            EditMessageText::new()
                .chat_id(user_id)
                .text("❌ Заявка в чат отклонена")
                .message_id(message_id),
        )
        .await?;

        bot.send(DeclineChatJoinRequest::new(chat_id, user_id))
            .await?;

        bot.send(
            BanChatMember::new(chat_id, user_id)
                .until_date(get_current_datetime().and_utc().timestamp() + 300),
        )
        .await?;
    }

    Ok(())
}

pub async fn garant_call_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    let summon_id = args.require("summon_id");
    let user_id = call.from.id;
    let chat_id = call.message.as_ref().map(|m| m.chat().id()).unwrap_or(0);

    let cached_data = match SUMMON_CACHE.get(summon_id).await {
        Some(d) => d,
        None => {
            bot.send(
                AnswerCallbackQuery::new(call.id.clone())
                    .text("❌ Данные созыва устарели или кнопка больше не активна.")
                    .show_alert(true),
            )
            .await?;

            if let Some(msg) = call.message {
                bot.send(
                    EditMessageReplyMarkup::new()
                        .chat_id(msg.chat().id())
                        .message_id(msg.message_id()),
                )
                .await?;
            }
            return Ok(());
        }
    };

    let is_author = user_id == cached_data.creator_id;
    let is_admin = ADMIN_IDS.contains(&user_id);

    let garant_repo = GarantRepo::new(db.clone());
    let is_garant = garant_repo.get(user_id).await.is_ok();

    if !is_author && !is_admin && !is_garant {
        bot.send(
            AnswerCallbackQuery::new(call.id)
                .text(
                    "❌ Только автор созыва, гаранты и администрация могут удалить эти сообщения!",
                )
                .show_alert(true),
        )
        .await?;
        return Ok(());
    }

    match bot
        .send(DeleteMessages::new(chat_id, cached_data.msg_ids.clone()))
        .await
    {
        Ok(_) => {
            SUMMON_CACHE.remove(summon_id).await;
            bot.send(AnswerCallbackQuery::new(call.id).text("✅ Созыв удален."))
                .await?;
        }
        Err(_) => {
            bot.send(
                AnswerCallbackQuery::new(call.id)
                    .text("⚠️ Не удалось удалить сообщения.")
                    .show_alert(true),
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn repeat_reg_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
    let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);

    let cfg = get_config();
    let iris_api = IrisAPI::new(cfg.iris_api_id, cfg.iris_api_token.clone());
    let user_reg = iris_api.get_user_reg(user_id).await.ok();

    if let Some(reg_data) = user_reg {
        if reg_data.get("error").is_some() {
            bot.send(
                AnswerCallbackQuery::new(call.id)
                    .text("ℹ️ Вы не выдали боту права на просмотр даты регистрации в Iris")
                    .show_alert(true),
            )
            .await?;
        } else {
            let reg_timestamp = reg_data["result"].as_i64().unwrap_or(0);
            let reg_timestamp_seconds = reg_timestamp / 1000;

            let now_msk = Utc::now().with_timezone(&Moscow);
            let year_ago_msk = now_msk - Duration::days(365);
            let reg_date_msk = Moscow
                .timestamp_opt(reg_timestamp_seconds, 0)
                .single()
                .expect("Invalid ts");

            if reg_date_msk < year_ago_msk {
                bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
                    .await?;
                let captcha_repo = CaptchaRepo::new(db);
                let _ = captcha_repo.insert(chat_id, user_id).await;

                if let Some(msg) = call.message {
                    bot.send(
                        EditMessageText::new()
                            .chat_id(msg.chat().id())
                            .message_id(msg.message_id())
                            .text("✅ Заявка в чат принята!"),
                    )
                    .await?;
                }
            } else {
                bot.send(DeclineChatJoinRequest::new(chat_id, user_id))
                    .await?;
                if let Some(msg) = call.message {
                    bot.send(EditMessageText::new().chat_id(msg.chat().id()).message_id(msg.message_id()).text("❌ Заявка в чат отклонена, вы не проходите по минимальной дате регистрации в Iris")).await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn unmute_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
    let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
    let message_id = args.require("message_id").parse::<i64>().unwrap_or(0);
    let message_id = Some(message_id);

    if !ADMIN_IDS.contains(&call.from.id) {
        bot.send(
            AnswerCallbackQuery::new(call.id.clone())
                .text("У вас недостаточно прав")
                .show_alert(true),
        )
        .await?;
        return Ok(());
    }

    let member = bot.send(GetChatMember::new(chat_id, user_id)).await?;

    match member {
        ChatMember::Restricted(_) => {
            let permissions = ChatPermissions {
                can_send_messages: Some(true),
                can_send_audios: Some(true),
                can_send_documents: Some(true),
                can_send_photos: Some(true),
                can_send_videos: Some(true),
                can_send_video_notes: Some(true),
                can_send_polls: Some(true),
                can_send_other_messages: Some(true),
                can_add_web_page_previews: Some(true),
                can_react_to_messages: Some(true),
                can_edit_tag: Some(true),
                can_change_info: Some(true),
                can_invite_users: Some(true),
                can_pin_messages: Some(true),
                can_send_voice_notes: Some(true),
                can_manage_topics: Some(true),
            };

            bot.send(RestrictChatMember::new(chat_id, user_id, permissions))
                .await?;

            let user_mention = get_user_mention(
                member.id(),
                member.username().map(|s| s.to_string()),
                member.first_name().parse()?,
            );
            let admin_mention = get_user_mention(
                call.from.id,
                call.from.username.map(|s| s.to_string()),
                call.from.first_name.parse()?,
            );
            let text = format!(
                "{} C {} сняли ограничения\n{} Модератор: {}",
                Emoji::Information,
                user_mention,
                Emoji::Human,
                admin_mention
            );

            if let Some(msg_id) = message_id
                && msg_id != 0
            {
                bot.send(
                    SendMessage::new(chat_id, text)
                        .reply_parameters(ReplyParameters::new(msg_id))
                        .link_preview_options(LinkPreviewOptions::new().is_disabled(true))
                        .parse_mode("HTML"),
                )
                .await?;
            } else {
                let message_id = match call.message {
                    Some(m) => m.message_id(),
                    None => return Ok(()),
                };

                bot.send(
                    SendMessage::new(chat_id, text)
                        .reply_parameters(ReplyParameters::new(message_id))
                        .link_preview_options(LinkPreviewOptions::new().is_disabled(true))
                        .parse_mode("HTML"),
                )
                .await?;
            }
        }
        _ => {
            bot.send(
                AnswerCallbackQuery::new(call.id.clone())
                    .text("Пользователь не лишен права слова")
                    .show_alert(true),
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn ban_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
    let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
    let message_id = args.require("message_id").parse::<i64>().unwrap_or(0);
    let message_id = Some(message_id);

    if !ADMIN_IDS.contains(&call.from.id) {
        bot.send(
            AnswerCallbackQuery::new(call.id.clone())
                .text("У вас недостаточно прав")
                .show_alert(true),
        )
        .await?;
        return Ok(());
    }

    let member = bot.send(GetChatMember::new(chat_id, user_id)).await?;

    match member {
        ChatMember::Kicked(_) => {
            bot.send(
                AnswerCallbackQuery::new(call.id.clone())
                    .text("Пользователь уже исключен из чата")
                    .show_alert(true),
            )
            .await?;
        }
        _ => {
            bot.send(BanChatMember::new(chat_id, user_id)).await?;

            let user_mention = get_user_mention(
                member.id(),
                member.username().map(|s| s.to_string()),
                member.first_name().parse()?,
            );
            let admin_mention = get_user_mention(
                call.from.id,
                call.from.username.map(|s| s.to_string()),
                call.from.first_name.parse()?,
            );
            let text = format!(
                "{} Пользователь {} исключен из чата\n{} Модератор: {}",
                Emoji::Information,
                user_mention,
                Emoji::Human,
                admin_mention
            );

            if let Some(msg_id) = message_id
                && msg_id != 0
            {
                bot.send(
                    SendMessage::new(chat_id, text)
                        .reply_parameters(ReplyParameters::new(msg_id))
                        .link_preview_options(LinkPreviewOptions::new().is_disabled(true))
                        .parse_mode("HTML"),
                )
                .await?;
            } else {
                let message_id = match call.message {
                    Some(m) => m.message_id(),
                    None => return Ok(()),
                };

                bot.send(
                    SendMessage::new(chat_id, text)
                        .reply_parameters(ReplyParameters::new(message_id))
                        .link_preview_options(LinkPreviewOptions::new().is_disabled(true))
                        .parse_mode("HTML"),
                )
                .await?;
            }
        }
    }

    Ok(())
}
