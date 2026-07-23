use telers::Bot;
use telers::methods::{LeaveChat, SendMessage};

pub const ADMIN_CHAT_ID: i64 = -1003904096608;
pub const DUEL_CHAT_ID: i64 = -1001876817712;
pub const GARANT_CHAT_ID: i64 = -1002393805826;
pub const SCAM_CHANNEL_ID: i64 = -1003979922414;
pub const GL_ADMINS: [i64; 3] = [1830362280, 8630742541, 1396129644];
pub const ADMIN_IDS: [i64; 23] = [
    1830362280, 5785884253, 8577420947, 1396129644, 5448752141, 5253969011, 5971869071,
    7868116959, 1979940844, 8630742541, 8138413942, 6842411953, 7595142206, 8003158848,
    7693221405, 7129739921, 8083769211, 5509186306, 7674435738, 5234395382, 6362416136,
    6755121814, 8424758022
];
const ALLOWED_CHATS: [i64; 7] = [
    -1001876817712,
    -1002393805826,
    -1001664794867,
    -1001986907414,
    -1003979922414,
    -1003904096608,
    -1002635887529
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
        let _ = bot.send(SendMessage::new(chat_id, "👋 Я выхожу")).await;
        let _ = bot.send(LeaveChat::new(chat_id)).await;
        return false;
    }
    true
}
