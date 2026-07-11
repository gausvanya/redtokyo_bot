use std::time::Instant;

use crate::bot::enums::tg_emoji::Emoji;
use crate::bot::filters::command::ParsedCommand;
use crate::bot::methods::message::MessageMethods;
use telers::{Bot, Extension, types::Message};

pub async fn ping_command_handler(
    bot: Bot,
    msg: Message,
    Extension(_): Extension<ParsedCommand>,
) -> anyhow::Result<()> {
    let start_ping = Instant::now();

    let message_text = format!("{} Проверка пинга...", Emoji::PingPong);
    let send_msg = bot
        .send(MessageMethods::send(&msg).text(message_text))
        .await?;

    let ping = start_ping.elapsed().as_millis();

    let (status, emoji) = match ping {
        0..50 => ("скоростной", Emoji::Rocket),
        50..200 => ("быстрый", Emoji::Car),
        200..500 => ("медленный", Emoji::Runner),
        _ => ("ужасный", Emoji::Turtle),
    };

    let message_text = format!(
        "{} <b>ПОНГ!</b>\n\n\
        {} Ответ: <b>{} | {} ms.</b>",
        Emoji::PingPong,
        emoji,
        status,
        ping
    );

    bot.send(MessageMethods::edit(&send_msg).text(message_text))
        .await?;
    Ok(())
}
