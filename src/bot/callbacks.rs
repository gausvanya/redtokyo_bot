use crate::bot::utils::datetime::get_current_datetime;
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Moscow;
use sea_orm::DatabaseConnection;

use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::libs::iris_api::IrisAPI;
use crate::bot::utils::chat::ADMIN_IDS;
use crate::bot::utils::user::get_user_mention;
use crate::database::cache::SUMMON_CACHE;
use crate::database::repo::captcha_repo::CaptchaRepo;
use crate::database::repo::garant_repo::GarantRepo;
use telers::event::simple::HandlerResult;
use telers::methods::{
    AnswerCallbackQuery, ApproveChatJoinRequest, BanChatMember, DeclineChatJoinRequest,
    DeleteMessages, EditMessageReplyMarkup, GetChatMember, RestrictChatMember
};
use telers::types::{
    CallbackQuery, ChatMember, ChatPermissions, ReplyParameters,
};
use telers::{Bot, Extension};
use crate::bot::methods::message::MessageMethods;

pub async fn captcha_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    unsafe {
        let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
        let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
        let code = args.require("code");
        let message = call.message.unwrap_unchecked();

        if code == "3" {
            let captcha_repo = CaptchaRepo::new(db.clone());
            let _ = captcha_repo.insert(chat_id, user_id).await;
            
            bot.send(
                MessageMethods::edit(&message)
                    .text("✅ Заявка в чат принята!")
                    .message_id(message.message_id())
            ).await?;
            bot.send(ApproveChatJoinRequest::new(chat_id, user_id)).await?;
        } else {
            bot.send(
                MessageMethods::edit(&message)
                    .text("❌ Заявка в чат отклонена")
                    .message_id(message.message_id())
            ).await?;

            bot.send(DeclineChatJoinRequest::new(chat_id, user_id)).await?;
            bot.send(
                BanChatMember::new(chat_id, user_id)
                    .until_date(get_current_datetime().and_utc().timestamp() + 300),
            ).await?;
        }
        Ok(())
    }
}

pub async fn garant_call_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    unsafe {
        let summon_id = args.require("summon_id");
        let user_id = call.from.id;
        let message = call.message.unwrap_unchecked();
        let chat_id = message.chat().id();

        let cached_data = match SUMMON_CACHE.get(summon_id).await {
            Some(d) => d,
            None => {
                bot.send(
                    AnswerCallbackQuery::new(call.id.clone())
                        .text("❌ Данные созыва устарели или кнопка больше не активна.")
                        .show_alert(true),
                ).await?;

                bot.send(
                    EditMessageReplyMarkup::new()
                        .chat_id(chat_id)
                        .message_id(message.message_id()),
                ).await?;

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
            ).await?;
            return Ok(());
        }

        bot.send(DeleteMessages::new(chat_id, cached_data.msg_ids.clone())).await?;
        Ok(())
    }
}

pub async fn repeat_reg_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    unsafe {
        let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
        let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
        let message = call.message.unwrap_unchecked();

        let iris_api = IrisAPI::new();
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
                    .unwrap_unchecked();

                if reg_date_msk < year_ago_msk {
                    bot.send(ApproveChatJoinRequest::new(chat_id, user_id)).await?;

                    let captcha_repo = CaptchaRepo::new(db);
                    captcha_repo.insert(chat_id, user_id).await?;

                    bot.send(
                        MessageMethods::edit(&message)
                            .text("✅ Заявка в чат принята!")
                            .message_id(message.message_id())
                    ).await?;
                } else {
                    bot.send(DeclineChatJoinRequest::new(chat_id, user_id)).await?;

                    bot.send(
                        MessageMethods::edit(&message)
                            .text("❌ Заявка в чат отклонена, вы не проходите по минимальной дате регистрации в Iris")
                            .message_id(message.message_id())
                    ).await?;
                }
            }
        }
        Ok(())
    }
}

pub async fn unmute_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    unsafe {
        let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
        let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
        let message_id = args.require("message_id").parse::<i64>().unwrap_or(0);
        let message = call.message.unwrap_unchecked();

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

                bot.send(RestrictChatMember::new(chat_id, user_id, permissions)).await?;

                let user_mention = get_user_mention(
                    member.id(),
                    member.username(),
                    member.first_name().parse()?,
                );
                let admin_mention = get_user_mention(
                    call.from.id,
                    call.from.username.as_deref(),
                    call.from.first_name.parse()?,
                );
                let text = format!(
                    "{} C {} сняли ограничения\n{} Модератор: {}",
                    Emoji::Information,
                    user_mention,
                    Emoji::Human,
                    admin_mention
                );

                if message_id != 0 {
                    bot.send(
                        MessageMethods::send(&message)
                            .text(text)
                            .reply_parameters(ReplyParameters::new().message_id(message_id))
                    ).await?;
                } else {
                    bot.send(MessageMethods::send(&message).text(text)).await?;
                }
            }
            _ => {
                bot.send(
                    AnswerCallbackQuery::new(call.id.clone())
                        .text("Пользователь не лишен права слова")
                        .show_alert(true),
                ).await?;
            }
        }
        Ok(())
    }
}

pub async fn ban_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(args): Extension<ParsedCommand>,
) -> HandlerResult {
    unsafe {
        let chat_id = args.require("chat_id").parse::<i64>().unwrap_or(0);
        let user_id = args.require("user_id").parse::<i64>().unwrap_or(0);
        let message_id = args.require("message_id").parse::<i64>().unwrap_or(0);
        let message = call.message.unwrap_unchecked();

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
                    member.username(),
                    member.first_name().parse()?,
                );
                let admin_mention = get_user_mention(
                    call.from.id,
                    call.from.username.as_deref(),
                    call.from.first_name.parse()?,
                );
                let text = format!(
                    "{} Пользователь {} исключен из чата\n{} Модератор: {}",
                    Emoji::Information,
                    user_mention,
                    Emoji::Human,
                    admin_mention
                );

                if message_id != 0 {
                    bot.send(
                        MessageMethods::send(&message)
                            .text(text)
                            .reply_parameters(ReplyParameters::new().message_id(message_id))
                    ).await?;
                } else {
                    bot.send(MessageMethods::send(&message).text(text)).await?;
                }
            }
        }
        Ok(())
    }
}
