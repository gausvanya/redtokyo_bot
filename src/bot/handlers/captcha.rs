use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Moscow;
use sea_orm::DatabaseConnection;
use telers::{
    Bot, Extension,
    methods::{
        ApproveChatJoinRequest, BanChatMember, DeclineChatJoinRequest, GetChat, GetUserGifts,
        SendMessage,
    },
    types::{ChatJoinRequest, ChatMemberUpdated, LinkPreviewOptions, OwnedGift},
};

use crate::{
    bot::{
        enums::tg_emoji::Emoji,
        keyboards::{captcha_keyboard, repeat_reg_keyboard},
        libs::iris_api::{IrisAPI, IrisApiError},
        utils::{
            chat::{ADMIN_CHAT_ID, GARANT_CHAT_ID, PR_CHAT_ID},
            datetime::get_current_datetime,
            user::UserMention,
        },
    },
    database::repo::captcha_repo::CaptchaRepo,
};

pub async fn captcha_chat_join_request_handler(
    bot: Bot,
    event: ChatJoinRequest,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let chat_id = event.chat.id();
    let user_id = event.from.id;

    if chat_id == PR_CHAT_ID {
        return Ok(());
    }

    let captcha_repo = CaptchaRepo::new(db.clone());

    let captcha_user = captcha_repo
        .get(chat_id, user_id)
        .await?;

    if captcha_user.is_some() {
        bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
            .await?;
        return Ok(());
    }

    let bot_clone = bot.clone();
    let db_clone = db.clone();

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

        let repo = CaptchaRepo::new(db_clone);

        match repo
            .get(chat_id, user_id)
            .await
        {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = bot_clone
                    .send(DeclineChatJoinRequest::new(chat_id, user_id))
                    .await;
                let _ = bot_clone
                    .send(
                        BanChatMember::new(chat_id, user_id).until_date(
                            get_current_datetime()
                                .and_utc()
                                .timestamp()
                                + 300,
                        ),
                    )
                    .await;
            }
            Err(e) => tracing::error!("Ошибка проверки капчи в spawn: {:?}", e),
        }
    });

    if chat_id == GARANT_CHAT_ID {
        let gifts = bot
            .send(GetUserGifts::new(user_id))
            .await?;
        let mut regular_count: i16 = 0;
        let mut nft_count: i16 = 0;

        for gift in gifts.gifts.iter() {
            match gift {
                OwnedGift::Regular(gift) => {
                    if let Some(user) = &gift.sender_user
                        && !user.is_bot
                    {
                        regular_count += 1;
                    }
                }
                OwnedGift::Unique(_) => {
                    nft_count += 1;
                }
                OwnedGift::Unknown(_) => return Ok(()),
            }
        }

        if nft_count >= 1 || regular_count >= 3 {
            if bot
                .send(ApproveChatJoinRequest::new(chat_id, user_id))
                .await
                .is_err()
            {
                return Ok(());
            }

            captcha_repo
                .insert(chat_id, user_id)
                .await?;
        } else {
            let iris_api = IrisAPI::new();

            match iris_api
                .get_user_reg(user_id)
                .await
            {
                Ok(user_reg) => {
                    let reg_timestamp = user_reg["result"]
                        .as_i64()
                        .unwrap_or(0);
                    let now_msk = Utc::now().with_timezone(&Moscow);
                    let year_ago_msk = now_msk - Duration::days(365);

                    let reg_timestamp_seconds = reg_timestamp / 1000;
                    let reg_date_msk = match Moscow
                        .timestamp_opt(reg_timestamp_seconds, 0)
                        .single()
                    {
                        Some(dt) => dt,
                        None => {
                            tracing::error!(
                                "Iris API вернул некорректный timestamp: {}",
                                reg_timestamp
                            );
                            return Ok(());
                        }
                    };

                    if reg_date_msk < year_ago_msk {
                        if bot
                            .send(ApproveChatJoinRequest::new(chat_id, user_id))
                            .await
                            .is_err()
                        {
                            return Ok(());
                        }
                        let _ = bot
                            .send(SendMessage::new(user_id, "✅ Заявка в чат принята!"))
                            .await;
                        captcha_repo
                            .insert(chat_id, user_id)
                            .await?;
                    } else {
                        if bot
                            .send(DeclineChatJoinRequest::new(chat_id, user_id))
                            .await
                            .is_err()
                        {
                            return Ok(());
                        }
                        let _ = bot
                            .send(SendMessage::new(
                                user_id,
                                "❌ Заявка в чат отклонена, вы не проходите по минимальной дате \
                                 регистрации в Iris",
                            ))
                            .await;
                    }
                }
                Err(IrisApiError::Api {
                    code: 403,
                    ..
                }) => {
                    let _ = bot
                        .send(
                            SendMessage::new(
                                user_id,
                                format!(
                                    "{} Бот запрашивает разрешение на получение информации о дате регистрации в Iris: \
                        <a href='https://t.me/iris_bs_bot?start=request_rights_7635712622_reg'>перейти</a>",
                                    Emoji::Information
                                ),
                            )
                                .parse_mode("HTML")
                                .reply_markup(repeat_reg_keyboard(chat_id, user_id)),
                        )
                        .await;
                }
                Err(err) => {
                    tracing::error!("Ошибка при запросе к Iris API: {:?}", err);
                    return Err(err.into());
                }
            }
        }
    } else {
        let user_mention = event.from.mention();
        bot.send(
            SendMessage::new(
                user_id,
                format!(
                    "{} {}\nПройди проверку на бота, нажав кнопку, соответствующую эмодзи \
                     'Курицы' ниже {}",
                    Emoji::Bot,
                    user_mention,
                    Emoji::ArrowDown
                ),
            )
            .parse_mode("HTML")
            .reply_markup(captcha_keyboard(chat_id, user_id)),
        )
        .await?;
    }

    Ok(())
}

pub async fn chat_member_updated_handler(
    bot: Bot,
    event: ChatMemberUpdated,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let user = event
        .new_chat_member
        .user();
    let chat_id = event.chat.id();

    if chat_id == PR_CHAT_ID {
        return Ok(());
    }

    let captcha_repo = CaptchaRepo::new(db);

    let is_captcha = captcha_repo
        .get(chat_id, user.id)
        .await?;

    if is_captcha.is_none() {
        let chat = bot
            .send(GetChat::new(chat_id))
            .await?;

        if !chat
            .join_by_request()
            .unwrap_or(false)
        {
            return Ok(());
        }

        let until_date = (Utc::now() + Duration::minutes(5)).timestamp();
        bot.send(BanChatMember::new(chat_id, user.id).until_date(until_date))
            .await?;

        bot.send(
            SendMessage::new(
                ADMIN_CHAT_ID,
                format!(
                    "{} {} зашел в чат в обход системы проверок, исключаю...\n{} Чат: {}",
                    Emoji::Warning,
                    user.mention(),
                    Emoji::Balloon,
                    event
                        .chat
                        .title()
                        .unwrap_or_default()
                ),
            )
            .parse_mode("HTML")
            .link_preview_options(LinkPreviewOptions::new().is_disabled(true)),
        )
        .await?;
    }
    Ok(())
}
