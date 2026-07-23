use crate::bot::callbacks;
use crate::bot::filters::callback::CallbackFilter;
use crate::bot::filters::command::CommandFilter;
use crate::bot::filters::regexes;
use telers::enums::ChatMemberType;
use telers::filters::ChatMemberUpdated;
use telers::{event::telegram::Handler, Router};

mod bot_welcome;
mod captcha;
mod duel;
mod garant;
mod ping;
mod scam_base;
mod verbal_warns;
mod db_update;

#[inline]
pub fn register_routers() -> Router {
    Router::new("main")
        .on_message(|observer| {
            observer.registers([
                Handler::new(ping::ping_command_handler)
                    .filter(CommandFilter::new(regexes::re_ping())),
                Handler::new(duel::duel_command_handler)
                    .filter(CommandFilter::new(regexes::re_duel())),
                Handler::new(garant::set_garant_command_handler)
                    .filter(CommandFilter::new(regexes::re_set_garant())),
                Handler::new(garant::remove_garant_command_handler)
                    .filter(CommandFilter::new(regexes::re_remove_garant())),
                Handler::new(garant::garant_list_command_handler)
                    .filter(CommandFilter::new(regexes::re_list_garants())),
                Handler::new(garant::garant_call_command_handler)
                    .filter(CommandFilter::new(regexes::re_call_garants())),
                Handler::new(verbal_warns::set_warn_command_handler)
                    .filter(CommandFilter::new(regexes::re_set_warn())),
                Handler::new(verbal_warns::remove_warn_command_handler)
                    .filter(CommandFilter::new(regexes::re_remove_warn())),
                Handler::new(verbal_warns::list_warns_command_handler)
                    .filter(CommandFilter::new(regexes::re_list_warns())),
                Handler::new(scam_base::set_scam_command_handler)
                    .filter(CommandFilter::new(regexes::re_set_scam())),
                Handler::new(scam_base::remove_scam_command_handler)
                    .filter(CommandFilter::new(regexes::re_remove_scam())),
                Handler::new(scam_base::reason_scam_command_handler)
                    .filter(CommandFilter::new(regexes::re_reason_scam())),
                Handler::new(scam_base::file_id_command_handler)
                    .filter(CommandFilter::new(regexes::re_file_id())),
                Handler::new(duel::minimal_rate_duel_command_handler)
                    .filter(CommandFilter::new(regexes::re_minimal_rate()))
            ])
        })
        .on_chat_join_request(|observer| {
            observer.register(Handler::new(captcha::captcha_chat_join_request_handler))
        })
        .on_my_chat_member(|observer| {
            observer.register(Handler::new(bot_welcome::bot_welcome_handler))
        })
        .on_chat_member(|observer| {
            observer.register(Handler::new(captcha::chat_member_updated_handler)
                .filter(
                    ChatMemberUpdated::new(ChatMemberType::Member).old(ChatMemberType::Left)
                )
            )
        })
        .on_callback_query(|observer| {
            observer.registers([
                Handler::new(callbacks::captcha_callback_handler)
                    .filter(CallbackFilter::new(regexes::re_callback_captcha())),
                Handler::new(callbacks::garant_call_callback_handler)
                    .filter(CallbackFilter::new(regexes::re_del_sum())),
                Handler::new(callbacks::repeat_reg_callback_handler)
                    .filter(CallbackFilter::new(regexes::re_repeat_reg())),
                Handler::new(callbacks::unmute_callback_handler)
                    .filter(CallbackFilter::new(regexes::re_unmute())),
                Handler::new(callbacks::ban_callback_handler)
                    .filter(CallbackFilter::new(regexes::re_ban())),
            ])
        })
}
