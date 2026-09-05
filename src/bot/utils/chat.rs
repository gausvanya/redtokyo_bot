use sea_orm::DatabaseConnection;
use telers::{
    Bot,
    methods::{LeaveChat, SendMessage},
    types::ChatPermissions,
};

use crate::database::repo::allowed_chats_repo::AllowedChatsRepo;

pub const ADMIN_CHAT_ID: i64 = -1003904096608;
pub const DUEL_CHAT_ID: i64 = -1001876817712;
pub const GARANT_CHAT_ID: i64 = -1002393805826;
pub const SCAM_CHANNEL_ID: i64 = -1003979922414;
pub const PR_CHAT_ID: i64 = -1002635887529;
pub const GL_ADMINS: [i64; 5] = [1830362280, 8630742541, 1396129644, 8083769211, 8577420947];
pub const ALLOWED_BOT_IDS: [i64; 10] = [
    8289185888, 8670571630, 6212219963, 6775391315, 6032895492, 1559501630, 5788046441, 5014831088,
    650863105, 8377231659,
];

pub const ALLOWED_URLS: [&str; 5] = [
    "https://t.me/+M3fsh0ruW75mODFi",
    "https://t.me/+XNXqm9WklD05MjE6",
    "https://t.me/+HcBJcoox5SYxMTgy",
    "https://t.me/+rDfQrZagTJw1MzYy",
    "https://t.me/+xWMOTmTObWxkOGM6",
];

pub async fn is_allowed_chat(
    bot: &Bot,
    chat_id: i64,
    db: DatabaseConnection,
) -> anyhow::Result<bool> {
    let allowed_chats_repo = AllowedChatsRepo::new(db);
    let is_allowed_chat = allowed_chats_repo
        .get(chat_id)
        .await?;

    if is_allowed_chat.is_none() {
        let _ = bot
            .send(SendMessage::new(chat_id, "👋 Я выхожу, мне тут не место."))
            .await;
        let _ = bot
            .send(LeaveChat::new(chat_id))
            .await;
        return Ok(false);
    }
    Ok(true)
}

pub fn muted_permissions() -> ChatPermissions {
    ChatPermissions {
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
    }
}

pub fn full_permissions() -> ChatPermissions {
    ChatPermissions {
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
    }
}
