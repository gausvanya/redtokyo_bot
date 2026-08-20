use telers::{
    Bot,
    methods::{LeaveChat, SendMessage},
    types::ChatPermissions,
};

pub const ADMIN_CHAT_ID: i64 = -1003904096608;
pub const DUEL_CHAT_ID: i64 = -1001876817712;
pub const GARANT_CHAT_ID: i64 = -1002393805826;
pub const SCAM_CHANNEL_ID: i64 = -1003979922414;
pub const PR_CHAT_ID: i64 = -1002635887529;
pub const GL_ADMINS: [i64; 4] = [1830362280, 8630742541, 1396129644, 8083769211];

pub const ADMIN_IDS: [i64; 19] = [
    1830362280, 5785884253, 8577420947, 1396129644, 5448752141, 5253969011, 7868116959, 8630742541,
    8138413942, 7595142206, 8003158848, 7693221405, 8083769211, 6755121814, 48292668, 222457737,
    5971869071, 7667509370, 6613866139
];
const ALLOWED_CHATS: [i64; 7] = [
    -1001876817712,
    -1002393805826,
    -1001664794867,
    -1001986907414,
    -1003979922414,
    -1003904096608,
    -1002635887529,
];
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

pub async fn is_allowed_chat(bot: &Bot, chat_id: i64) -> bool {
    if !ALLOWED_CHATS.contains(&chat_id) {
        let _ = bot
            .send(SendMessage::new(chat_id, "👋 Я выхожу"))
            .await;
        let _ = bot
            .send(LeaveChat::new(chat_id))
            .await;
        return false;
    }
    true
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
