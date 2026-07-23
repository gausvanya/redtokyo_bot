use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::enums::user_type::UserIdentity;
use crate::bot::methods::message::MessageMethods;
use crate::bot::utils::user::get_user_info;
use crate::database::repo::user_repo::UserRepo;
use sea_orm::DatabaseConnection;
use telers::Bot;
use telers::types::Message;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: Option<String>,
    pub full_name: String,
}

pub struct GetUserInfo {
    user: Option<String>,
    user_repo: UserRepo,
    bot: Bot,
}

impl GetUserInfo {
    pub fn new(user: Option<String>, db: &DatabaseConnection, bot: Bot) -> Self {
        Self {
            user,
            user_repo: UserRepo::new(db.clone()),
            bot,
        }
    }

    pub async fn resolve(&self, msg: &Message) -> anyhow::Result<Option<UserInfo>> {
        if self.user.is_none()
            && let Some(reply_msg) = msg.reply_to_message()
        {
            let (id, username, full_name) = get_user_info(reply_msg);
            return Ok(Some(UserInfo {
                id,
                username: username.map(|b| b.into_string()),
                full_name: full_name.into_string(),
            }));
        }

        if let Some(user_val) = &self.user {
            let parsed_id = user_val.parse::<i64>().ok().or_else(|| {
                user_val
                    .strip_prefix('_')
                    .and_then(|s| format!("-{s}").parse::<i64>().ok())
            });

            let user_obj = if let Some(user_id) = parsed_id {
                self.user_repo.get(UserIdentity::Id(user_id)).await?
            } else {
                self.user_repo
                    .get(UserIdentity::Username(user_val.to_string()))
                    .await?
            };

            if let Some(user) = user_obj {
                return Ok(Some(UserInfo {
                    id: user.id,
                    username: user.username,
                    full_name: user.full_name,
                }));
            }
        }

        let _ = self
            .bot
            .send(MessageMethods::send(msg).text(format!(
                "{} Telegram API вернул ошибку:\n\
            Указан неверный идентификатор пользователя или мне ничего не известно о нем",
                Emoji::Warning
            )))
            .await;

        Ok(None)
    }
}
