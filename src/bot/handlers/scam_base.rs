use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::filters::get_user::GetUserInfo;
use crate::bot::methods::message::MessageMethods;
use crate::bot::middlewares::media_group::MediaKind;
use crate::bot::utils::chat::{ADMIN_IDS, SCAM_CHANNEL_ID};
use crate::bot::utils::user::{get_user_info, get_user_mention};
use crate::database::cache::MEDIA_GROUP_CACHE;
use crate::database::models::scam_base;
use crate::database::repo::scam_base::ScamBaseRepo;
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use telers::event::simple::HandlerResult;
use telers::methods::{DeleteMessage, SendDocument, SendMediaGroup, SendPhoto, SendVideo};
use telers::types::{
    FileId, InputMedia, InputMediaPhoto, InputMediaVideo, Message, ReplyParameters,
};
use telers::{Bot, Extension};

const RED_STATUS: &str = "AgACAgIAAyEFAASglsVWAAIpMmoPFXVoP7Ws4wMmPaHPp2ki4FLKAAKIIGsbaKZ5SDqWL07xu2FLAAgBAAMCAAN3AAceBA";
const YELLOW_STATUS: &str = "AgACAgIAAyEFAASglsVWAAIpMGoPFW2XejOE4n-j0vPWQbGTiLX1AAKGIGsbaKZ5SBpainAvbA5jAAgBAAMCAAN3AAceBA";
const GREEN_STATUS: &str = "AgACAgIAAyEFAASglsVWAAIpMWoPFXKT-_9D7GVoKC_D5XJSozBwAAKHIGsbaKZ5SDreok4QrSaYAAgBAAMCAAN3AAceBA";

// const RED_STATUS: &str =
//     "AgACAgIAAyEFAAMBC-_NCAACAThqS2MTsI_lB0Iv6pl9_B_fxHoOZwACRhdrG-YHWUp7ZREzH2BC3QEAAwIAA3cAAzwE";
// const YELLOW_STATUS: &str =
//     "AgACAgIAAyEFAAMBC-_NCAACATlqS2MTB5Flyj6LtpTjWPAh_ImI7wACRxdrG-YHWUo10_mKv79swwEAAwIAA3cAAzwE";
// const GREEN_STATUS: &str =
//     "AgACAgIAAyEFAAMBC-_NCAACATpqS2MTgiLtw4T-9NTzUA7jJfN_aAACSBdrG-YHWUp7Hl080SOfnAEAAwIAA3cAAzwE";

pub async fn set_scam_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin = get_user_info(&msg);

    if !ADMIN_IDS.contains(&admin.0) {
        return Ok(());
    }

    let (user, reason) = (args.get("user"), args.require("reason").to_string());

    let reply_msg = if let Some(r) = msg.reply_to_message() {
        r
    } else {
        bot.send(MessageMethods::send(&msg).text(format!(
            "<i>{} Команда должна быть вызвана в ответ на смс с выданным наказанием</i>",
            Emoji::Exclamation
        )))
        .await?;

        return Ok(());
    };
    let reply_id = reply_msg.message_id();

    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);
        let admin_mention = get_user_mention(admin.0, admin.1.as_deref(), admin.2.to_string());
        let url = format!(
            "https://t.me/c/{}/{}",
            msg.chat().id().to_string().replace("-100", ""),
            reply_id
        );

        let message_text = format!(
            "<i>#бан\n\
         {} <b>Пользователь {} (<code>@{}</code>) находится в скам базе проекта 'RedTokyo'</b>\n\
         {} <b>Причина:</b> {}\n\
         {} <b>Модератор:</b> {}\n\n\
         {} <b><a href='{}'>Перейти к смс бана</a></b></i>",
            Emoji::Exclamation,
            user_mention,
            user.id,
            Emoji::Balloon,
            reason,
            Emoji::Human,
            admin_mention,
            Emoji::ArrowRight,
            url
        );

        let sent_msg: Option<Message>;

        let mut file_ids = Vec::new();

        if let Some(mg_id) = msg.media_group_id()
            && let Some(mutex) = MEDIA_GROUP_CACHE.get(&mg_id.to_string()).await
        {
            file_ids = mutex.lock().await.clone();
        }

        if !file_ids.is_empty() {
            let media: Vec<InputMedia> = file_ids
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let caption = if i == 0 {
                        Some(message_text.clone())
                    } else {
                        None
                    };

                    match item.kind {
                        MediaKind::Photo => {
                            let mut p = InputMediaPhoto::new(FileId::new(item.file_id.clone()));
                            if let Some(c) = caption {
                                p = p.caption(c).parse_mode("HTML");
                            }
                            p.into()
                        }
                        MediaKind::Video => {
                            let mut v = InputMediaVideo::new(FileId::new(item.file_id.clone()));
                            if let Some(c) = caption {
                                v = v.caption(c).parse_mode("HTML");
                            }
                            v.into()
                        }
                    }
                })
                .collect();

            let sent_messages = bot
                .send(SendMediaGroup::new(SCAM_CHANNEL_ID, media))
                .await?;
            sent_msg = sent_messages.into_iter().next();
        } else {
            let photo = msg.photo().and_then(|p| p.last());
            let video = msg.video();
            let document = msg.document();

            if let Some(photo) = photo {
                let req = SendPhoto::new(SCAM_CHANNEL_ID, FileId::new(photo.file_id.clone()))
                    .caption(message_text.clone())
                    .parse_mode("HTML");
                sent_msg = Some(bot.send(req).await?);
            } else if let Some(video) = video {
                let req = SendVideo::new(SCAM_CHANNEL_ID, FileId::new(video.file_id.clone()))
                    .caption(message_text.clone())
                    .parse_mode("HTML");
                sent_msg = Some(bot.send(req).await?);
            } else if let Some(doc) = document {
                let req = SendDocument::new(SCAM_CHANNEL_ID, FileId::new(doc.file_id.clone()))
                    .caption(message_text.clone())
                    .parse_mode("HTML");
                sent_msg = Some(bot.send(req).await?);
            } else {
                let req = MessageMethods::send(&msg)
                    .chat_id(SCAM_CHANNEL_ID)
                    .reply_parameters_option(None::<ReplyParameters>)
                    .text(message_text.clone());
                sent_msg = Some(bot.send(req).await?);
            }
        }

        if let Some(s_msg) = sent_msg {
            let s_chat_id_str = s_msg.chat().id().to_string();
            let s_url_chat_id = s_chat_id_str.strip_prefix("-100").unwrap_or(&s_chat_id_str);
            let baza_url = format!("https://t.me/c/{}/{}", s_url_chat_id, s_msg.message_id());

            let reply_text = format!(
                "<i>{} Пользователь {} занесен в скам базу проекта 'RedTokyo'\n\
            <b><a href='{}'>Перейти к смс скам-базы</a></b></i>",
                Emoji::Human,
                user_mention,
                baza_url
            );

            let reply_req = MessageMethods::send(&msg)
                .text(reply_text)
                .reply_parameters_option(None::<ReplyParameters>);

            let reply_msg = bot.send(reply_req).await?;

            let scam_base_repo = ScamBaseRepo::new(db);

            let result = scam_base_repo.get(user.id).await;

            match result {
                Ok(Some(i)) => {
                    let model = i.0;

                    let channel_chat_id = model.channel_chat_id;
                    let channel_message_id = model.channel_message_id;

                    let mut active_model = model.into_active_model();
                    active_model.chat_id = Set(msg.chat().id());
                    active_model.status = Set(true);
                    active_model.message_id = Set(reply_msg.message_id());
                    active_model.admin_id = Set(admin.0);
                    active_model.channel_message_id = Set(s_msg.message_id());
                    active_model.reason = Set(reason);
                    scam_base_repo.update(active_model).await?;

                    bot.send(DeleteMessage::new(channel_chat_id, channel_message_id)).await?;
                }
                _ => {
                    scam_base_repo
                        .insert(
                            msg.chat().id(),
                            user.id,
                            reply_msg.message_id(),
                            admin.0,
                            SCAM_CHANNEL_ID,
                            s_msg.message_id(),
                            reason,
                            true,
                        )
                        .await?;
                }
            }

        }
    }
    Ok(())
}

pub async fn remove_scam_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin = get_user_info(&msg);

    if !ADMIN_IDS.contains(&admin.0) {
        return Ok(());
    }

    let (user, command) = (args.get("user"), args.require("command"));
    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        let scam_base_repo = ScamBaseRepo::new(db);

        let scam_base = scam_base_repo.get(user.id).await?;

        let msg_text = match scam_base {
            Some((scam, _)) => {
                if command.to_lowercase().contains("ошибка") {
                    if !GL_ADMINS.contains(&admin.0) {
                        bot.send(MessageMethods::send(&msg)
                            .text(format!(
                                "{} У вас недостаточно прав для выполнения данной команды.",
                                Emoji::Exclamation)
                            )
                        ).await?;

                        return Ok(())
                    }

                    let channel_chat_id = scam.channel_chat_id;
                    let channel_message_id = scam.channel_message_id;

                    let _ = bot.send(DeleteMessage::new(channel_chat_id, channel_message_id)).await;
                    scam_base_repo.delete(scam).await?;

                    format!(
                        "{} Пользователь {} удален из скам базы проекта 'RedTokyo' без пометки о вносе",
                        Emoji::Human,
                        user_mention
                    )
                } else {
                    let mut active_model: scam_base::ActiveModel = scam.into_active_model();
                    active_model.status = Set(false);
                    let _ = scam_base_repo.update(active_model).await;
                    format!(
                        "{} Пользователь {} удален из скам базы проекта 'RedTokyo'",
                        Emoji::Human,
                        user_mention
                    )
                }
            }
            _ => format!(
                "{} Пользователь {} отсутствует в скам базе проекта 'RedTokyo'",
                Emoji::Human,
                user_mention
            ),
        };

        bot.send(MessageMethods::send(&msg).text(msg_text)).await?;
    }
    Ok(())
}

pub async fn reason_scam_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let user = args.get("user");
    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        let scam_base_repo = ScamBaseRepo::new(db);

        let scam_base = scam_base_repo.get(user.id).await?;

        let (photo, msg_text) = match scam_base {
            Some((scam_base, Some(admin_user))) => {
                let admin_mention =
                    get_user_mention(admin_user.id, admin_user.username.as_deref(), admin_user.full_name);

                let status = if scam_base.status {
                    "находится"
                } else {
                    "находился"
                };
                let scam_url = format!(
                    "https://t.me/c/{}/{}",
                    scam_base.chat_id.to_string().replace("-100", ""),
                    scam_base.message_id
                );
                let photo_id = if scam_base.status {
                    RED_STATUS
                } else {
                    YELLOW_STATUS
                };

                (
                    FileId::new(photo_id),
                    format!(
                        "{} <i><b>Пользователь {} (<code>@{}</code>) {} в скам базе проекта 'RedTokyo'</b>\n\
                {} <b>Причина:</b> {}\n\
                {} <b>Модератор:</b> {}\n\n\
                {} <b><a href='{}'>Перейти к смс скам-базы</a></b></i>",
                        Emoji::Exclamation,
                        user_mention,
                        user.id,
                        status,
                        Emoji::Balloon,
                        scam_base.reason,
                        Emoji::Human,
                        admin_mention,
                        Emoji::ArrowRight,
                        scam_url
                    ),
                )
            }
            _ => (
                FileId::new(GREEN_STATUS),
                format!(
                    "{} Пользователь {} отсутствует в скам базе проекта 'RedTokyo'",
                    Emoji::Human,
                    user_mention
                ),
            ),
        };

        bot.send(
            SendPhoto::new(msg.chat().id(), photo)
                .caption(msg_text)
                .parse_mode("HTML"),
        )
        .await?;
    }
    Ok(())
}

pub async fn file_id_command_handler(bot: Bot, msg: Message) -> HandlerResult {
    let reply_msg = if let Some(r) = msg.reply_to_message() {
        r
    } else {
        bot.send(
            MessageMethods::send(&msg)
                .text("<i>❗️ Команда должна быть вызвана в ответ на смс с выданным фото.</i>"),
        )
        .await?;
        return Ok(());
    };

    if let Some(photo) = reply_msg.photo().and_then(|p| p.last()) {
        let text = format!("<i>💬 Файл ид: <code>{}</code></i>", photo.file_id);

        bot.send(MessageMethods::send(&msg).text(text).parse_mode("HTML"))
            .await?;
    } else {
        bot.send(MessageMethods::send(&msg).text("<i>❗️ Это сообщение не содержит фото.</i>"))
            .await?;
    }

    Ok(())
}
