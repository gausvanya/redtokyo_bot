use telers::{
    enums::ParseMode,
    methods::{EditMessageText, SendMessage},
    types::{LinkPreviewOptions, Message, ReplyParameters},
};
pub struct MessageMethods;

impl MessageMethods {
    #[inline]
    pub fn send(msg: &Message) -> SendMessage {
        SendMessage::new(msg.chat().id(), "")
            .message_thread_id_option(msg.message_thread_id())
            .parse_mode(ParseMode::HTML)
            .link_preview_options(LinkPreviewOptions {
                is_disabled: Some(true),
                ..Default::default()
            })
            .reply_parameters(ReplyParameters::new(msg.message_id()))
    }
    #[inline]
    pub fn edit(msg: &Message) -> EditMessageText {
        EditMessageText::new()
            .chat_id(msg.chat().id())
            .message_id(msg.message_id())
            .parse_mode(ParseMode::HTML)
    }
}
