use telers::types::{MaybeInaccessibleMessage, Message};
use telers::{
    enums::ParseMode,
    methods::{EditMessageText, SendMessage},
    types::{LinkPreviewOptions, ReplyParameters},
};

pub struct MessageMethods;

impl MessageMethods {
    #[inline]
    pub fn send<T: IntoMessage>(msg: T) -> SendMessage {
        let m = msg.into_message();

        SendMessage::new(m.chat().id(), "")
            .message_thread_id_option(m.message_thread_id())
            .parse_mode(ParseMode::HTML)
            .link_preview_options(LinkPreviewOptions {
                is_disabled: Some(true),
                ..Default::default()
            })
            .reply_parameters(ReplyParameters::new().message_id(m.message_id()))
    }

    #[inline]
    pub fn edit<T: IntoMessage>(msg: T) -> EditMessageText {
        let m = msg.into_message();

        EditMessageText::new()
            .chat_id(m.chat().id())
            .message_id(m.message_id())
            .parse_mode(ParseMode::HTML)
    }
}


pub trait IntoMessage {
    fn into_message(self) -> MaybeInaccessibleMessage;
}

impl IntoMessage for Message {
    fn into_message(self) -> MaybeInaccessibleMessage {
        MaybeInaccessibleMessage::Message(self)
    }
}

impl IntoMessage for &Message {
    fn into_message(self) -> MaybeInaccessibleMessage {
        MaybeInaccessibleMessage::Message(self.clone())
    }
}

impl IntoMessage for Box<MaybeInaccessibleMessage> {
    fn into_message(self) -> MaybeInaccessibleMessage {
        *self
    }
}

impl IntoMessage for &Box<MaybeInaccessibleMessage> {
    fn into_message(self) -> MaybeInaccessibleMessage {
        (**self).clone()
    }
}

impl IntoMessage for MaybeInaccessibleMessage {
    fn into_message(self) -> MaybeInaccessibleMessage {
        self
    }
}
