use std::fmt;
use strum::EnumProperty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumProperty)]
#[rustfmt::skip]
pub enum Emoji {
    #[strum(props(id = "5269563867305879894", char = "🏓"))] PingPong,
    #[strum(props(id = "5188481279963715781", char = "🚀"))] Rocket,
    #[strum(props(id = "5418176697889466758", char = "🏎"))] Car,
    #[strum(props(id = "5312079370611860224", char = "🏃"))] Runner,
    #[strum(props(id = "5393142059370559102", char = "🐢"))] Turtle,
    #[strum(props(id = "5372981976804366741", char = "🤖"))] Bot,
    #[strum(props(id = "5465300082628763143", char = "💬"))] Balloon,
    #[strum(props(id = "5447644880824181073", char = "⚠️"))] Warning,
    #[strum(props(id = "5440660757194744323", char = "‼️"))] Bangbang,
    #[strum(props(id = "5406745015365943482", char = "⬇️"))] ArrowDown,
    #[strum(props(id = "5199750217586459631", char = "📄"))] FacingUp,
    #[strum(props(id = "5215344475039084599", char = "📣"))] Megaphone,
    #[strum(props(id = "5366231379136763743", char = "🔘"))] RadioButton,
    #[strum(props(id = "5274099962655816924", char = "❗"))] Exclamation,
    #[strum(props(id = "5458789419014182183", char = "️👤"))] Human,
    #[strum(props(id = "5436276364384677952", char = "➡️"))] ArrowRight,
    #[strum(props(id = "5334544901428229844", char = "ℹ️"))] Information,


}

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.get_str("id").unwrap_or("");
        let char = self.get_str("char").unwrap_or("");

        write!(f, "<tg-emoji emoji-id='{}'>{}</tg-emoji>", id, char)
    }
}
