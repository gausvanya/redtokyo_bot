use crate::bot::filters::regexes::RE_INVITE;
use crate::bot::utils::chat::{ALLOWED_BOT_IDS, ALLOWED_URLS};
use crate::bot::utils::user::get_user_info;
use crate::database::cache::RAID_CACHE;
use regex::Regex;
use telers::Bot;
use telers::client::Session;
use telers::methods::GetChatMember;
use telers::types::{ChatMember, Message};

pub struct AntispamFilter;

impl AntispamFilter {
    fn is_spam_link(&self, msg: &Message) -> bool {
        if let Some(u) = msg.from()
            && u.is_bot
        {
            return false;
        }

        let text = msg.text().or(msg.caption()).unwrap_or("");
        let re = RE_INVITE.get_or_init(|| Regex::new(r"t\.me/\+\w+").unwrap());

        for mat in re.find_iter(text) {
            if !ALLOWED_URLS.contains(&mat.as_str()) {
                return true;
            }
        }
        false
    }

    pub async fn check_bot_access<Client>(&self, bot: &Bot<Client>, msg: &Message) -> bool
    where
        Client: Session,
    {
        let user = match msg.from() {
            Some(u) if u.is_bot => u,
            _ => return true,
        };

        if ALLOWED_BOT_IDS.contains(&user.id) {
            return true;
        }

        let member_result = bot.send(GetChatMember::new(msg.chat().id(), user.id)).await;

        match member_result {
            Ok(ChatMember::Left(_)) | Ok(ChatMember::Kicked(_)) => false,
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn is_raid_safe(&self, msg: &Message) -> bool {
        if msg.media_group_id().is_some() {
            return false;
        }

        if let Some(u) = msg.from()
            && u.is_bot
        {
            return false;
        }

        let user_id = get_user_info(msg).0;
        let chat_id = msg.chat().id();
        let key = format!("{}:{}", chat_id, user_id);

        let now = msg.date();

        let mut timestamps = RAID_CACHE.get(&key).await.unwrap_or_else(Vec::new);
        timestamps.push(now);
        timestamps.retain(|&ts| ts > (now - 8));

        RAID_CACHE.insert(key, timestamps.clone()).await;

        timestamps.len() > 10
    }

    pub async fn check<C>(&self, bot: &Bot<C>, msg: &Message) -> (bool, &'static str, Vec<i64>)
    where
        C: Session + Send + Sync + 'static,
    {
        if self.is_spam_link(msg) {
            return (false, "spam", vec![msg.message_id()]);
        }
        if self.is_raid_safe(msg).await {
            let user_id = get_user_info(msg).0;
            let chat_id = msg.chat().id();
            let key = format!("{}:{}", chat_id, user_id);
            let timestamps = RAID_CACHE.get(&key).await.unwrap_or_else(Vec::new);

            return (false, "raid", timestamps);
        }
        if !self.check_bot_access(bot, msg).await {
            return (false, "bot", vec![msg.message_id()]);
        }
        (true, "null", Vec::new())
    }
}
