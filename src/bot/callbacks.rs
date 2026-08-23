use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Moscow;
use sea_orm::DatabaseConnection;
use telers::{
    Bot, Extension,
    methods::{
        AnswerCallbackQuery, ApproveChatJoinRequest, BanChatMember, DeclineChatJoinRequest,
        DeleteMessages, EditMessageReplyMarkup, GetChat, GetChatMember, RestrictChatMember,
    },
    types::{CallbackQuery, ChatMember, ReplyParameters},
};

use crate::{
    bot::{
        enums::tg_emoji::Emoji,
        filters::command::ParsedCommand,
        keyboards::repeat_novo_reg_keyboard,
        libs::iris_api::{IrisAPI, IrisApiError},
        methods::message::MessageMethods,
        utils::{
            chat::{ADMIN_CHAT_ID, full_permissions},
            datetime::get_current_datetime,
            user::{UserMention, get_user_mention},
        },
    },
    database::{
        cache::SUMMON_CACHE,
        repo::{admins_repo::AdminRepo, captcha_repo::CaptchaRepo, garant_repo::GarantRepo},
    },
};

pub async fn captcha_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    let Some(message) = call.message else {
        return Ok(());
    };

    let (chat_id, user_id, code) = unsafe {
        (
            args.require("chat_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("user_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("code"),
        )
    };

    if code == "3" {
        let iris_api = IrisAPI::new();
        match iris_api
            .get_user_reg(user_id)
            .await
        {
            Ok(value) => {
                let reg_timestamp = value["result"]
                    .as_i64()
                    .unwrap_or(0);
                let reg_timestamp_seconds = reg_timestamp / 1000;

                let reg_date = Moscow
                    .timestamp_opt(reg_timestamp_seconds, 0)
                    .single()
                    .unwrap();

                let now_msk = Utc::now().with_timezone(&Moscow);
                let three_months_ago = now_msk - Duration::days(90);
                let is_suspicious = reg_date > three_months_ago;

                if is_suspicious {
                    let formatted_date = reg_date
                        .format("%d.%m.%Y %H:%M")
                        .to_string();
                    let duration = now_msk.signed_duration_since(reg_date);
                    let total_days = duration.num_days();
                    let months = total_days / 30;
                    let remaining_days_after_months = total_days % 30;

                    let weeks = duration.num_weeks();
                    let days = duration.num_days() % 7;

                    let diff_str = if months > 0 {
                        if remaining_days_after_months > 0 {
                            format!("{} мес. {} дн.", months, remaining_days_after_months)
                        } else {
                            format!("{} мес.", months)
                        }
                    } else if weeks > 0 {
                        if days > 0 {
                            format!("{} нед. {} дн.", weeks, days)
                        } else {
                            format!("{} нед.", weeks)
                        }
                    } else if total_days > 0 {
                        format!("{} дн.", total_days)
                    } else if duration.num_hours() > 0 {
                        format!("{} ч.", duration.num_hours())
                    } else if duration.num_minutes() > 0 {
                        format!("{} мин.", duration.num_minutes())
                    } else if duration.num_seconds() > 0 {
                        format!("{} сек.", duration.num_seconds())
                    } else {
                        "дата не распознана".to_string()
                    };

                    let reg_info_formatted = format!("{} ({})", formatted_date, diff_str);

                    let current_chat = bot
                        .send(GetChat::new(chat_id))
                        .await?;
                    let user_mention = call.from.mention();

                    let admin_text = format!(
                        "{} Участник {} (<code>@{}</code>) вступил в чат {}\n{} Имеет регистрацию \
                         в ирисе: {}",
                        Emoji::Information,
                        user_mention,
                        user_id,
                        current_chat
                            .title()
                            .unwrap_or(""),
                        Emoji::Date,
                        reg_info_formatted,
                    );

                    let captcha_repo = CaptchaRepo::new(db.clone());
                    let _ = captcha_repo
                        .insert(chat_id, user_id)
                        .await;

                    let _ = bot
                        .send(
                            MessageMethods::send(&message)
                                .chat_id(ADMIN_CHAT_ID)
                                .text(admin_text)
                                .reply_parameters_option(None::<ReplyParameters>),
                        )
                        .await;
                }

                bot.send(
                    MessageMethods::edit(&message)
                        .text("✅ Заявка обработана успешно, добро пожаловать в чат!")
                        .message_id(message.message_id()),
                )
                .await?;
                bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
                    .await?;
            }
            Err(_) => {
                let _ = bot
                    .send(
                        MessageMethods::edit(&message).chat_id(user_id).message_id(message.message_id()).text(
                            format!(
                                "{} Бот запрашивает разрешение на получение информации о дате регистрации в Iris: \
                        <a href='https://t.me/iris_black_bot?start=request_rights_7635712622_reg'>перейти</a>",
                                Emoji::Information
                            ),
                        ).reply_markup(repeat_novo_reg_keyboard(chat_id, user_id)),
                    ).await?;
            }
        }
    } else {
        bot.send(
            MessageMethods::edit(&message)
                .text("❌ Заявка в чат отклонена")
                .message_id(message.message_id()),
        )
        .await?;

        bot.send(DeclineChatJoinRequest::new(chat_id, user_id))
            .await?;
        bot.send(
            BanChatMember::new(chat_id, user_id).until_date(
                get_current_datetime()
                    .and_utc()
                    .timestamp()
                    + 300,
            ),
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
) -> anyhow::Result<()> {
    let Some(message) = call.message else {
        return Ok(());
    };

    let summon_id = args.require("summon_id");
    let user_id = call.from.id;
    let chat_id = message.chat().id();

    let cached_data = match SUMMON_CACHE
        .get(summon_id)
        .await
    {
        Some(d) => d,
        None => {
            bot.send(
                AnswerCallbackQuery::new(call.id.clone())
                    .text("❌ Данные созыва устарели или кнопка больше не активна.")
                    .show_alert(true),
            )
            .await?;

            bot.send(
                EditMessageReplyMarkup::new()
                    .chat_id(chat_id)
                    .message_id(message.message_id()),
            )
            .await?;

            return Ok(());
        }
    };

    let is_author = user_id == cached_data.creator_id;

    let admin_repo = AdminRepo::new(db.clone());

    let is_admin = admin_repo
        .get(user_id)
        .await?
        .is_some();

    let garant_repo = GarantRepo::new(db.clone());
    let is_garant = garant_repo
        .get(user_id)
        .await
        .is_ok();

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

    bot.send(DeleteMessages::new(
        chat_id,
        cached_data
            .msg_ids
            .clone(),
    ))
    .await?;
    Ok(())
}

pub async fn repeat_reg_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    let Some(message) = call.message else {
        return Ok(());
    };

    let (chat_id, user_id) = unsafe {
        (
            args.require("chat_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("user_id")
                .parse::<i64>()
                .unwrap_unchecked(),
        )
    };

    let iris_api = IrisAPI::new();

    match iris_api
        .get_user_reg(user_id)
        .await
    {
        Ok(user_reg) => {
            let reg_timestamp = user_reg["result"]
                .as_i64()
                .unwrap_or_default();
            let reg_timestamp_seconds = reg_timestamp / 1000;

            let now_msk = Utc::now().with_timezone(&Moscow);
            let year_ago_msk = now_msk - Duration::days(365);
            let reg_date_msk = unsafe {
                Moscow
                    .timestamp_opt(reg_timestamp_seconds, 0)
                    .single()
                    .unwrap_unchecked()
            };

            if reg_date_msk < year_ago_msk {
                bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
                    .await?;

                let captcha_repo = CaptchaRepo::new(db);
                captcha_repo
                    .insert(chat_id, user_id)
                    .await?;

                bot.send(
                    MessageMethods::edit(&message)
                        .text("✅ Заявка в чат принята!")
                        .message_id(message.message_id()),
                )
                .await?;
            } else {
                bot.send(DeclineChatJoinRequest::new(chat_id, user_id))
                    .await?;

                bot.send(
                    MessageMethods::edit(&message)
                        .text(
                            "❌ Заявка в чат отклонена, вы не проходите по минимальной дате \
                             регистрации в Iris",
                        )
                        .message_id(message.message_id()),
                )
                .await?;
            }
        }
        Err(IrisApiError::Api {
            code: 403,
            ..
        }) => {
            bot.send(
                AnswerCallbackQuery::new(call.id)
                    .text("ℹ️ Вы не выдали боту права на просмотр даты регистрации в Iris")
                    .show_alert(true),
            )
            .await?;
        }
        Err(err) => {
            tracing::error!("Ошибка при запросе к Iris API: {:?}", err);
            return Err(err.into());
        }
    }
    Ok(())
}

pub async fn repeat_novo_reg_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    let Some(message) = call.message else {
        return Ok(());
    };

    let (chat_id, user_id) = unsafe {
        (
            args.require("chat_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("user_id")
                .parse::<i64>()
                .unwrap_unchecked(),
        )
    };

    let iris_api = IrisAPI::new();

    match iris_api
        .get_user_reg(user_id)
        .await
    {
        Ok(value) => {
            let reg_timestamp = value["result"]
                .as_i64()
                .unwrap_or(0);
            let reg_timestamp_seconds = reg_timestamp / 1000;

            let reg_date = Moscow
                .timestamp_opt(reg_timestamp_seconds, 0)
                .single()
                .unwrap();

            let now_msk = Utc::now().with_timezone(&Moscow);
            let three_months_ago = now_msk - Duration::days(90);
            let is_suspicious = reg_date > three_months_ago;

            if is_suspicious {
                let formatted_date = reg_date
                    .format("%d.%m.%Y %H:%M")
                    .to_string();
                let duration = now_msk.signed_duration_since(reg_date);
                let total_days = duration.num_days();
                let months = total_days / 30;
                let remaining_days_after_months = total_days % 30;

                let weeks = duration.num_weeks();
                let days = duration.num_days() % 7;

                let diff_str = if months > 0 {
                    if remaining_days_after_months > 0 {
                        format!("{} мес. {} дн.", months, remaining_days_after_months)
                    } else {
                        format!("{} мес.", months)
                    }
                } else if weeks > 0 {
                    if days > 0 {
                        format!("{} нед. {} дн.", weeks, days)
                    } else {
                        format!("{} нед.", weeks)
                    }
                } else if total_days > 0 {
                    format!("{} дн.", total_days)
                } else if duration.num_hours() > 0 {
                    format!("{} ч.", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{} мин.", duration.num_minutes())
                } else if duration.num_seconds() > 0 {
                    format!("{} сек.", duration.num_seconds())
                } else {
                    "дата не распознана".to_string()
                };

                let reg_info_formatted = format!("{} ({})", formatted_date, diff_str);

                let current_chat = bot
                    .send(GetChat::new(chat_id))
                    .await?;

                let user_mention = call.from.mention();

                let admin_text = format!(
                    "{} Участник {} (<code>@{}</code>) вступил в чат {}\n{} Имеет регистрацию в \
                     ирисе: {}",
                    Emoji::Information,
                    user_mention,
                    user_id,
                    current_chat
                        .title()
                        .unwrap_or(""),
                    Emoji::Date,
                    reg_info_formatted,
                );

                let _ = bot
                    .send(
                        MessageMethods::send(&message)
                            .chat_id(ADMIN_CHAT_ID)
                            .text(admin_text)
                            .reply_parameters_option(None::<ReplyParameters>),
                    )
                    .await;
            }

            let captcha_repo = CaptchaRepo::new(db);
            let _ = captcha_repo
                .insert(chat_id, user_id)
                .await;

            bot.send(
                MessageMethods::edit(&message)
                    .text("✅ Заявка обработана успешно, добро пожаловать в чат!")
                    .message_id(message.message_id()),
            )
            .await?;

            bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
                .await?;
        }
        Err(IrisApiError::Api {
            code: 403,
            ..
        }) => {
            bot.send(
                AnswerCallbackQuery::new(call.id)
                    .show_alert(true)
                    .text("ℹ️ Вы не выдали боту право на доступ к вашей регистрации в ирисе."),
            )
            .await?;
        }
        Err(err) => {
            tracing::error!("Ошибка при запросе к Iris API: {:?}", err);
            return Err(err.into());
        }
    }
    Ok(())
}

pub async fn unmute_callback_handler(
    bot: Bot,
    call: CallbackQuery,
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    let Some(message) = call.message else {
        return Ok(());
    };

    let (chat_id, user_id, message_id) = unsafe {
        (
            args.require("chat_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("user_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("message_id")
                .parse::<i64>()
                .unwrap_unchecked(),
        )
    };

    let admin_repo = AdminRepo::new(db);

    let is_admin = admin_repo
        .get(call.from.id)
        .await?
        .is_some();

    if !is_admin {
        bot.send(
            AnswerCallbackQuery::new(call.id.clone())
                .text("У вас недостаточно прав")
                .show_alert(true),
        )
        .await?;
        return Ok(());
    }

    let member = bot
        .send(GetChatMember::new(chat_id, user_id))
        .await?;

    match member {
        ChatMember::Restricted(_) => {
            let permissions = full_permissions();

            bot.send(RestrictChatMember::new(chat_id, user_id, permissions))
                .await?;

            let user_mention = get_user_mention(
                member.id(),
                member.username(),
                member
                    .first_name()
                    .parse()?,
            );
            let admin_mention = get_user_mention(
                call.from.id,
                call.from
                    .username
                    .as_deref(),
                call.from
                    .first_name
                    .parse()?,
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
                        .reply_parameters(ReplyParameters::new().message_id(message_id)),
                )
                .await?;
            } else {
                bot.send(MessageMethods::send(&message).text(text))
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
    Extension(db): Extension<DatabaseConnection>,
    Extension(args): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    let Some(message) = call.message else {
        return Ok(());
    };

    let (chat_id, user_id, message_id) = unsafe {
        (
            args.require("chat_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("user_id")
                .parse::<i64>()
                .unwrap_unchecked(),
            args.require("message_id")
                .parse::<i64>()
                .unwrap_unchecked(),
        )
    };

    let admin_repo = AdminRepo::new(db.clone());

    let is_admin = admin_repo
        .get(call.from.id)
        .await?
        .is_some();

    if !is_admin {
        bot.send(
            AnswerCallbackQuery::new(call.id.clone())
                .text("У вас недостаточно прав")
                .show_alert(true),
        )
        .await?;
        return Ok(());
    }

    let member = bot
        .send(GetChatMember::new(chat_id, user_id))
        .await?;

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
            bot.send(BanChatMember::new(chat_id, user_id))
                .await?;

            let user_mention = get_user_mention(
                member.id(),
                member.username(),
                member
                    .first_name()
                    .parse()?,
            );
            let admin_mention = get_user_mention(
                call.from.id,
                call.from
                    .username
                    .as_deref(),
                call.from
                    .first_name
                    .parse()?,
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
                        .reply_parameters(ReplyParameters::new().message_id(message_id)),
                )
                .await?;
            } else {
                bot.send(MessageMethods::send(&message).text(text))
                    .await?;
            }
        }
    }
    Ok(())
}
