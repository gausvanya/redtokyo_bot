use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::antispam::AntispamFilter;
use crate::bot::keyboards::antispam_keyboard;
use crate::bot::methods::message::MessageMethods;
use crate::bot::utils::chat::{ADMIN_CHAT_ID, ADMIN_IDS, SCAM_CHANNEL_ID};
use crate::bot::utils::user::{get_user_info, get_user_mention};
use telers::Request;
use telers::errors::EventErrorKind;
use telers::event::EventReturn;
use telers::methods::{DeleteMessages, RestrictChatMember, UnpinChatMessage};
use telers::middlewares::outer::{Middleware, MiddlewareResponse};
use telers::types::{ChatPermissions, ReplyParameters};

#[derive(Clone)]
pub struct AntispamMiddleware;

impl<Client> Middleware<Client> for AntispamMiddleware
where
    Client: Send + Sync + 'static + telers::client::Session,
{
    async fn call(
        &mut self,
        request: Request<Client>,
    ) -> Result<MiddlewareResponse<Client>, EventErrorKind> {
        unsafe {
            if let Some(msg) = request.update.message().or(request.update.edited_message()) {
                let chat_id = msg.chat().id();

                if let Some(sender) = msg.sender_chat()
                    && sender.id() == SCAM_CHANNEL_ID
                {
                    let _ = request
                        .bot
                        .send(UnpinChatMessage::new(chat_id).message_id(msg.message_id()))
                        .await;
                    return Ok((request, EventReturn::Skip));
                }

                let (user_id, username, full_name) = get_user_info(msg);

                if ADMIN_IDS.contains(&user_id) {
                    return Ok((request, EventReturn::default()));
                }

                let filter = AntispamFilter;
                let (is_passed, reason, messages) = filter.check(&request.bot, msg).await;

                if !is_passed {
                    let user_mention = get_user_mention(user_id, username.as_deref(), full_name.to_string());

                    let _ = request
                        .bot
                        .send(DeleteMessages::new(chat_id, messages))
                        .await;

                    let until_date = (chrono::Utc::now() + chrono::Duration::days(1)).timestamp();
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
                    if request
                        .bot
                        .send(
                            RestrictChatMember::new(chat_id, user_id, permissions)
                                .until_date(until_date),
                        )
                        .await
                        .is_err()
                    {
                        return Ok((request, EventReturn::default()));
                    }

                    let msg_text = match reason {
                        "spam" => {
                            format!(
                                "{} {} был ограничен на сутки\n\
                            {} Причина: Рассылка спам-ссылок.",
                                Emoji::Information,
                                user_mention,
                                Emoji::Balloon
                            )
                        }
                        "bot" => {
                            format!(
                                "{} Ограничен гостевой бот {} (<code>@{}</code>)\n\
                            {} Чат: {:?}\n\
                            {} Сообщение: {:?}",
                                Emoji::Bot,
                                user_mention,
                                user_id,
                                Emoji::Megaphone,
                                msg.chat().title().unwrap_or("Без названия"),
                                Emoji::Balloon,
                                msg.text().unwrap_or("")
                            )
                        }
                        "raid" => {
                            format!(
                                "📛 Сработал антирейд фильтр!\n\
                            {} Пользователь {} ограничен на сутки",
                                Emoji::Human,
                                user_mention
                            )
                        }
                        _ => return Ok((request, EventReturn::default())),
                    };

                    let sent_msg = request
                        .bot
                        .send(
                            MessageMethods::send(msg)
                                .chat_id(chat_id)
                                .reply_parameters_option(None::<ReplyParameters>)
                                .reply_markup(antispam_keyboard(chat_id, user_id, None))
                                .text(msg_text.clone()),
                        )
                        .await;

                    let msg_id = sent_msg.unwrap_unchecked().message_id();

                    let _ = request
                        .bot
                        .send(
                            MessageMethods::send(msg)
                                .chat_id(ADMIN_CHAT_ID)
                                .reply_parameters(ReplyParameters::new().chat_id(chat_id).message_id(msg_id))
                                .reply_markup(antispam_keyboard(chat_id, user_id, Some(msg_id)))
                                .text(msg_text),
                        )
                        .await;
                }
            }
            Ok((request, EventReturn::default()))
        }
    }
}
