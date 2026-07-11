use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::keyboards::{captcha_keyboard, repeat_reg_keyboard};
use crate::bot::libs::iris_api::IrisAPI;
use crate::bot::utils::chat::GARANT_CHAT_ID;
use crate::bot::utils::datetime::get_current_datetime;
use crate::bot::utils::user::get_user_mention;
use crate::config::get_config;
use crate::database::repo::captcha_repo::CaptchaRepo;
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Moscow;
use sea_orm::DatabaseConnection;
use telers::methods::{
    ApproveChatJoinRequest, BanChatMember, DeclineChatJoinRequest, GetUserGifts, SendMessage,
};
use telers::types::{ChatJoinRequest, OwnedGift};
use telers::{Bot, Extension};

pub async fn captcha_chat_join_request_handler(
    bot: Bot,
    event: ChatJoinRequest,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let chat_id = event.chat.id();
    let user_id = event.from.id;
    let captcha_repo = CaptchaRepo::new(db.clone());

    let captcha_user = captcha_repo.get(chat_id, user_id).await?;

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

        match repo.get(chat_id, user_id).await {
            Ok(Some(_)) => {}

            _ => {
                let _ = bot_clone
                    .send(DeclineChatJoinRequest::new(chat_id, user_id))
                    .await;

                let _ = bot_clone
                    .send(
                        BanChatMember::new(chat_id, user_id)
                            .until_date(get_current_datetime().and_utc().timestamp() + 300),
                    )
                    .await;
            }
        }
    });

    if chat_id == GARANT_CHAT_ID {
        let gifts = bot.send(GetUserGifts::new(user_id)).await?;
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
            }
        }

        if nft_count >= 1 || regular_count >= 3 {
            bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
                .await?;

            captcha_repo.insert(chat_id, user_id).await?;
        } else {
            let cfg = get_config();

            let iris_api = IrisAPI::new(cfg.iris_api_id, cfg.iris_api_token.clone());
            let user_reg = iris_api.get_user_reg(user_id).await?;

            if user_reg.get("error").is_some() {
                bot.send(SendMessage::new(user_id, format!(
                    "{} Бот запрашивает разрешение, на получение информации о дате регистрации в Iris: \
                        <a href='https://t.me/iris_bs_bot?start=request_rights_7635712622_reg'>перейти</a>",
                    Emoji::Information
                )).parse_mode("HTML").reply_markup(repeat_reg_keyboard(chat_id, user_id))).await?;
            } else {
                let reg_timestamp = user_reg["result"].as_i64().unwrap_or(0);
                let now_msk = Utc::now().with_timezone(&Moscow);

                let year_ago_msk = now_msk - Duration::days(365);

                let reg_timestamp_seconds = reg_timestamp / 1000;
                let reg_date_msk = Moscow
                    .timestamp_opt(reg_timestamp_seconds, 0)
                    .single()
                    .expect("Invalid timestamp");

                if reg_date_msk < year_ago_msk {
                    bot.send(ApproveChatJoinRequest::new(chat_id, user_id))
                        .await?;
                    bot.send(SendMessage::new(user_id, "✅ Заявка в чат принята!"))
                        .await?;
                    captcha_repo.insert(chat_id, user_id).await?;
                } else {
                    bot.send(DeclineChatJoinRequest::new(chat_id, user_id))
                        .await?;
                    bot.send(SendMessage::new(user_id, "❌ Заявка в чат отклонена, вы не проходите по минимальной дате регистрации в Iris")).await?;
                }
            }
        }
    } else {
        let user = event.from;
        let user_mention = get_user_mention(
            user.id,
            user.username.map(|s| s.to_string()),
            user.first_name.parse()?,
        );
        bot.send(
            SendMessage::new(
                user_id,
                format!(
                    "{} {}\n\
                    Пройди проверку на бота, нажав кнопку, соответствующую эмодзи 'Курицы' ниже {}",
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
