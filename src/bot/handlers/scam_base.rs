use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use telers::{
    Bot, Extension,
    event::simple::HandlerResult,
    methods::{DeleteMessage, DeleteMessages, SendDocument, SendMediaGroup, SendPhoto, SendVideo},
    types::{FileId, InputMedia, InputMediaPhoto, InputMediaVideo, Message, ReplyParameters},
};

use crate::{
    bot::{
        enums::tg_emoji::Emoji,
        filters::{command::ParsedCommand, get_user::GetUserInfo},
        methods::message::MessageMethods,
        middlewares::media_group::MediaKind,
        utils::{
            chat::{GL_ADMINS, SCAM_CHANNEL_ID},
            user::{get_user_info, get_user_mention},
        },
    },
    database::{
        cache::MEDIA_GROUP_CACHE,
        models::scam_base,
        repo::{admins_repo::AdminRepo, scam_base::ScamBaseRepo},
    },
};

const RED_STATUS: &str = "AgACAgIAAyEFAASglsVWAAIpMmoPFXVoP7Ws4wMmPaHPp2ki4FLKAAKIIGsbaKZ5SDqWL07xu2FLAAgBAAMCAAN3AAceBA";
const YELLOW_STATUS: &str = "AgACAgIAAyEFAASglsVWAAIpMGoPFW2XejOE4n-j0vPWQbGTiLX1AAKGIGsbaKZ5SBpainAvbA5jAAgBAAMCAAN3AAceBA";
const GREEN_STATUS: &str = "AgACAgIAAyEFAASglsVWAAIpMWoPFXKT-_9D7GVoKC_D5XJSozBwAAKHIGsbaKZ5SDreok4QrSaYAAgBAAMCAAN3AAceBA";

pub async fn set_scam_command_handler(
    bot: Bot,
    msg: Message,
    Extension(args): Extension<ParsedCommand>,
    Extension(db): Extension<DatabaseConnection>,
) -> anyhow::Result<()> {
    let admin = get_user_info(&msg);

    let admin_repo = AdminRepo::new(db.clone());

    if !admin_repo
        .get(admin.0)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let (user, reason) = (
        args.get("user"),
        args.require("reason")
            .to_string(),
    );

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
            msg.chat()
                .id()
                .to_string()
                .replace("-100", ""),
            reply_id
        );

        let message_text = format!(
            "<i>#бан\n{} <b>Пользователь {} (<code>@{}</code>) находится в скам базе проекта \
             'RedTokyo'</b>\n{} <b>Причина:</b> {}\n{} <b>Модератор:</b> {}\n\n{} <b><a \
             href='{}'>Перейти к смс бана</a></b></i>",
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

        let mut file_ids = Vec::new();

        if let Some(mg_id) = msg.media_group_id()
            && let Some(mutex) = MEDIA_GROUP_CACHE
                .get(&mg_id.to_string())
                .await
        {
            file_ids = mutex
                .lock()
                .await
                .items
                .clone();
        }

        let sent_msgs: Vec<Message> = if !file_ids.is_empty() {
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
                                p = p
                                    .caption(c)
                                    .parse_mode("HTML");
                            }
                            p.into()
                        }
                        MediaKind::Video => {
                            let mut v = InputMediaVideo::new(FileId::new(item.file_id.clone()));
                            if let Some(c) = caption {
                                v = v
                                    .caption(c)
                                    .parse_mode("HTML");
                            }
                            v.into()
                        }
                    }
                })
                .collect();

            Vec::from(
                bot.send(SendMediaGroup::new(SCAM_CHANNEL_ID, media))
                    .await?,
            )
        } else {
            let photo = msg
                .photo()
                .and_then(|p| p.last());
            let video = msg.video();
            let document = msg.document();

            let single_msg = if let Some(photo) = photo {
                let req = SendPhoto::new(SCAM_CHANNEL_ID, FileId::new(photo.file_id.clone()))
                    .caption(message_text.clone())
                    .parse_mode("HTML");
                bot.send(req).await?
            } else if let Some(video) = video {
                let req = SendVideo::new(SCAM_CHANNEL_ID, FileId::new(video.file_id.clone()))
                    .caption(message_text.clone())
                    .parse_mode("HTML");
                bot.send(req).await?
            } else if let Some(doc) = document {
                let req = SendDocument::new(SCAM_CHANNEL_ID, FileId::new(doc.file_id.clone()))
                    .caption(message_text.clone())
                    .parse_mode("HTML");
                bot.send(req).await?
            } else {
                let req = MessageMethods::send(&msg)
                    .chat_id(SCAM_CHANNEL_ID)
                    .reply_parameters_option(None::<ReplyParameters>)
                    .text(message_text.clone());
                bot.send(req).await?
            };

            vec![single_msg]
        };

        if let Some(s_msg) = sent_msgs
            .first()
            .cloned()
        {
            let s_chat_id_str = s_msg
                .chat()
                .id()
                .to_string();
            let s_url_chat_id = s_chat_id_str
                .strip_prefix("-100")
                .unwrap_or(&s_chat_id_str);
            let baza_url = format!("https://t.me/c/{}/{}", s_url_chat_id, s_msg.message_id());

            let reply_text = format!(
                "<i>{} Пользователь {} занесен в скам базу проекта 'RedTokyo'\n<b><a \
                 href='{}'>Перейти к смс скам-базы</a></b></i>",
                Emoji::Human,
                user_mention,
                baza_url
            );

            let reply_req = MessageMethods::send(&msg)
                .text(reply_text)
                .reply_parameters_option(None::<ReplyParameters>);

            let reply_msg = bot
                .send(reply_req)
                .await?;

            let channel_message_ids: Vec<i64> = sent_msgs
                .iter()
                .map(|m| m.message_id())
                .collect();

            let scam_base_repo = ScamBaseRepo::new(db);
            let result = scam_base_repo
                .get(user.id)
                .await;

            match result {
                Ok(Some(i)) => {
                    let model = i.0;
                    let channel_chat_id = model.channel_chat_id;
                    let old_ids = model
                        .channel_message_ids
                        .clone();

                    let mut active_model = model.into_active_model();
                    active_model.chat_id = Set(msg.chat().id());
                    active_model.status = Set(true);
                    active_model.message_id = Set(reply_msg.message_id());
                    active_model.admin_id = Set(admin.0);
                    active_model.channel_message_ids = Set(channel_message_ids);
                    active_model.reason = Set(reason);
                    scam_base_repo
                        .update(active_model)
                        .await?;

                    for old_id in old_ids {
                        let _ = bot
                            .send(DeleteMessage::new(channel_chat_id, old_id))
                            .await;
                    }
                }
                _ => {
                    scam_base_repo
                        .insert(
                            msg.chat().id(),
                            user.id,
                            reply_msg.message_id(),
                            admin.0,
                            SCAM_CHANNEL_ID,
                            channel_message_ids,
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

    let admin_repo = AdminRepo::new(db.clone());

    if !admin_repo
        .get(admin.0)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let (user, command) = (args.get("user"), args.require("command"));
    let user_obj = GetUserInfo::new(user.map(|s| s.to_string()), &db, bot.clone())
        .resolve(&msg)
        .await?;

    if let Some(user) = user_obj {
        let user_mention = get_user_mention(user.id, user.username.as_deref(), user.full_name);

        let scam_base_repo = ScamBaseRepo::new(db);

        let scam_base = scam_base_repo
            .get(user.id)
            .await?;

        let msg_text = match scam_base {
            Some((scam, _)) => {
                if command
                    .to_lowercase()
                    .contains("ошибка")
                {
                    if !GL_ADMINS.contains(&admin.0) {
                        bot.send(MessageMethods::send(&msg).text(format!(
                            "{} У вас недостаточно прав для выполнения данной команды.",
                            Emoji::Exclamation
                        )))
                        .await?;

                        return Ok(());
                    }

                    let channel_chat_id = scam.channel_chat_id;
                    let channel_message_ids = scam
                        .channel_message_ids
                        .clone();

                    let _ = bot
                        .send(DeleteMessages::new(channel_chat_id, channel_message_ids))
                        .await;
                    scam_base_repo
                        .delete(scam)
                        .await?;

                    format!(
                        "{} Пользователь {} удален из скам базы проекта 'RedTokyo' без пометки о \
                         вносе",
                        Emoji::Human,
                        user_mention
                    )
                } else {
                    let mut active_model: scam_base::ActiveModel = scam.into_active_model();
                    active_model.status = Set(false);
                    scam_base_repo
                        .update(active_model)
                        .await?;
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

        bot.send(MessageMethods::send(&msg).text(msg_text))
            .await?;
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

        let scam_base = scam_base_repo
            .get(user.id)
            .await?;

        let (photo, msg_text) = match scam_base {
            Some((scam_base, Some(admin_user))) => {
                let admin_mention = get_user_mention(
                    admin_user.id,
                    admin_user
                        .username
                        .as_deref(),
                    admin_user.full_name,
                );

                let status = if scam_base.status {
                    "находится"
                } else {
                    "находился"
                };
                let scam_url = format!(
                    "https://t.me/c/{}/{}",
                    scam_base
                        .channel_chat_id
                        .to_string()
                        .replace("-100", ""),
                    scam_base
                        .channel_message_ids
                        .first()
                        .copied()
                        .unwrap_or_default()
                );
                let photo_id = if scam_base.status {
                    RED_STATUS
                } else {
                    YELLOW_STATUS
                };

                (
                    FileId::new(photo_id),
                    format!(
                        "{} <i><b>Пользователь {} (<code>@{}</code>) {} в скам базе проекта \
                         'RedTokyo'</b>\n{} <b>Причина:</b> {}\n{} <b>Модератор:</b> {}\n\n{} \
                         <b><a href='{}'>Перейти к смс скам-базы</a></b></i>",
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

    if let Some(photo) = reply_msg
        .photo()
        .and_then(|p| p.last())
    {
        let text = format!("<i>💬 Файл ид: <code>{}</code></i>", photo.file_id);

        bot.send(
            MessageMethods::send(&msg)
                .text(text)
                .parse_mode("HTML"),
        )
        .await?;
    } else {
        bot.send(MessageMethods::send(&msg).text("<i>❗️ Это сообщение не содержит фото.</i>"))
            .await?;
    }

    Ok(())
}
