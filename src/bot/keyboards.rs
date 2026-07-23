use rand::seq::SliceRandom;
use telers::types::{CopyTextButton, InlineKeyboardButton, InlineKeyboardMarkup};

#[inline]
pub fn duel_chat_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::new("RT | Игры | Конкурсы")
            .url("https://t.me/+RKMmtfeDjGhhNzYy".to_string())
            .icon_custom_emoji_id("5443038326535759644")
            .style("primary"),
    ]])
}

#[inline]
pub fn garant_call_keyboard(summon_id: String) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::new("Удалить созыв")
            .callback_data(format!("del_sum:{}", summon_id))
            .icon_custom_emoji_id("5445267414562389170")
            .style("success"),
    ]])
}

#[inline]
pub fn repeat_reg_keyboard(chat_id: i64, user_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::new("Повторить попытку")
            .callback_data(format!("repeat_reg:{}:{}", chat_id, user_id))
            .style("success"),
    ]])
}

#[inline]
pub fn captcha_keyboard(chat_id: i64, user_id: i64) -> InlineKeyboardMarkup {
    let mut buttons = vec![
        InlineKeyboardButton::new(".")
            .callback_data(format!("captcha:{}:{}:1", chat_id, user_id))
            .icon_custom_emoji_id("5339237329292764502"),
        InlineKeyboardButton::new(".")
            .callback_data(format!("captcha:{}:{}:2", chat_id, user_id))
            .icon_custom_emoji_id("5337047059180566409"),
        InlineKeyboardButton::new(".")
            .callback_data(format!("captcha:{}:{}:3", chat_id, user_id))
            .icon_custom_emoji_id("5318986601441813952"),
        InlineKeyboardButton::new(".")
            .callback_data(format!("captcha:{}:{}:4", chat_id, user_id))
            .icon_custom_emoji_id("5357233044694508227"),
    ];

    let mut rng = rand::rng();
    buttons.shuffle(&mut rng);

    InlineKeyboardMarkup::new(vec![buttons])
}

#[inline]
pub fn duel_unmute_keyboard(
    chat_id: i64,
    user_id: i64,
    min_bet: i64,
    message_id: Option<i64>,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::new("Размутить")
                .callback_data(format!(
                    "unmute:{}:{:?}:{}",
                    chat_id,
                    message_id.unwrap_or(0),
                    user_id
                ))
                .icon_custom_emoji_id("5372800046284674872")
                .style("success"),
        ],
        vec![
            InlineKeyboardButton::new("Скопировать предупреждение").copy_text(CopyTextButton::new(
                format!("варн @{user_id}\nИгры от 50 голд."),
            )),
        ],
    ])
}

#[inline]
pub fn antispam_keyboard(
    chat_id: i64,
    user_id: i64,
    message_id: Option<i64>,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::new("Размутить")
                .callback_data(format!(
                    "unmute:{}:{:?}:{}",
                    chat_id,
                    message_id.unwrap_or(0),
                    user_id
                ))
                .icon_custom_emoji_id("5372800046284674872")
                .style("success"),
        ],
        vec![
            InlineKeyboardButton::new("Забанить")
                .callback_data(format!(
                    "ban:{}:{:?}:{}",
                    chat_id,
                    message_id.unwrap_or(0),
                    user_id
                ))
                .icon_custom_emoji_id("5472308992514464048")
                .style("success"),
        ],
    ])
}
