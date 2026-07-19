use regex::Regex;
use std::sync::OnceLock;

// COMMANDS REGEX STATIC
const PREFIX: &str = r#"(?:[.!/]\s?|сап\s?)?"#;
const USER_PATTERN: &str = r#"(?:\s+(?:https?://t\.me/|@|tg://(?:user\?id=|openmessage\?user_id=|resolve\?domain=)|<a\s+href=["']tg://user\?id=)?(?P<user>\d+|[a-zA-Z0-9_]{5,32})(?:["']?>[^<]*</a>)?)?"#;
static RE_PING: OnceLock<Regex> = OnceLock::new();
static RE_DUEL: OnceLock<Regex> = OnceLock::new();
static RE_SET_GARANT: OnceLock<Regex> = OnceLock::new();
static RE_REMOVE_GARANT: OnceLock<Regex> = OnceLock::new();
static RE_CALL_GARANTS: OnceLock<Regex> = OnceLock::new();
static RE_LIST_GARANTS: OnceLock<Regex> = OnceLock::new();
static RE_SET_WARN: OnceLock<Regex> = OnceLock::new();
static RE_REMOVE_WARN: OnceLock<Regex> = OnceLock::new();
static RE_LIST_WARNS: OnceLock<Regex> = OnceLock::new();
static RE_SET_SCAM: OnceLock<Regex> = OnceLock::new();
static RE_REMOVE_SCAM: OnceLock<Regex> = OnceLock::new();
static RE_REASON_SCAM: OnceLock<Regex> = OnceLock::new();
static RE_FILE_ID: OnceLock<Regex> = OnceLock::new();
static RE_MINIMAL_RATE: OnceLock<Regex> = OnceLock::new();


// CALLBACK REGEX STATIC
static RE_CALLBACK_CAPTCHA: OnceLock<Regex> = OnceLock::new();
static RE_DEL_SUM: OnceLock<Regex> = OnceLock::new();
static RE_REPEAT_REG: OnceLock<Regex> = OnceLock::new();
static RE_UNMUTE: OnceLock<Regex> = OnceLock::new();
static RE_BAN: OnceLock<Regex> = OnceLock::new();

// FILTERS
pub static RE_INVITE: OnceLock<Regex> = OnceLock::new();

// COMMANDS REGEX
#[inline]
pub fn re_ping() -> &'static Regex {
    RE_PING.get_or_init(|| {
        Regex::new(&format!(r"(?i)^{PREFIX}(?P<command>пинг|ping)(?:$|\n)")).unwrap()
    })
}

#[inline]
pub fn re_duel() -> &'static Regex {
    RE_DUEL.get_or_init(|| {
        Regex::new(
            r"(?i)^(?:[!./]|ириска?|ирис\s+)?(?P<command>кто дуэль|кто кубы|дуэль|кубы)\s*(?P<amount>\d+[кk]?)?(?:$|\n)"
        ).unwrap()
    })
}

#[inline]
pub fn re_set_garant() -> &'static Regex {
    RE_SET_GARANT.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)^{PREFIX}\+гарант{USER_PATTERN}\s*\n(?P<comment>[\s\S]+)"#
        ))
        .unwrap()
    })
}

#[inline]
pub fn re_remove_garant() -> &'static Regex {
    RE_REMOVE_GARANT.get_or_init(|| {
        Regex::new(&format!(r#"(?i)^{PREFIX}-гарант{USER_PATTERN}\s*(?:$|\n)"#)).unwrap()
    })
}

#[inline]
pub fn re_call_garants() -> &'static Regex {
    RE_CALL_GARANTS.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)^{PREFIX}созвать\s+гарантов(?:\s*\n(?P<reason>[\s\S]+))?"#
        ))
        .unwrap()
    })
}

#[inline]
pub fn re_list_garants() -> &'static Regex {
    RE_LIST_GARANTS
        .get_or_init(|| Regex::new(&format!(r#"(?i)^{PREFIX}(?:кто\s+)?гаранты(?:$|\n)"#)).unwrap())
}

#[inline]
pub fn re_set_warn() -> &'static Regex {
    RE_SET_WARN.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)^{PREFIX}\+уст{USER_PATTERN}\s*\n(?P<reason>[\s\S]+)"#
        ))
        .unwrap()
    })
}

#[inline]
pub fn re_remove_warn() -> &'static Regex {
    RE_REMOVE_WARN.get_or_init(|| {
        Regex::new(&format!(r#"(?i)^{PREFIX}-уст{USER_PATTERN}\s*(?:$|\n)"#)).unwrap()
    })
}

#[inline]
pub fn re_list_warns() -> &'static Regex {
    RE_LIST_WARNS.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)^{PREFIX}(?P<command>мои\s+усты|твои\s+усты){USER_PATTERN}\s*(?:$|\n)"#
        ))
        .unwrap()
    })
}

#[inline]
pub fn re_set_scam() -> &'static Regex {
    RE_SET_SCAM.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)^{PREFIX}\+скам база{USER_PATTERN}?\s*\n(?P<reason>[\s\S]+)"#
        ))
        .unwrap()
    })
}

#[inline]
pub fn re_remove_scam() -> &'static Regex {
    RE_REMOVE_SCAM.get_or_init(|| {
        Regex::new(&format!(
            r#"(?P<command>(?i)^{PREFIX}-скам база(?:\s+ошибка)?){USER_PATTERN}(?:$|\s+)"#
        ))
        .unwrap()
    })
}

#[inline]
pub fn re_reason_scam() -> &'static Regex {
    RE_REASON_SCAM.get_or_init(|| {
        Regex::new(&format!(r#"(?i)^{PREFIX}причина{USER_PATTERN}(?:$|\s+)"#)).unwrap()
    })
}

#[inline]
pub fn re_file_id() -> &'static Regex {
    RE_FILE_ID
        .get_or_init(|| Regex::new(&format!(r"(?i)^{PREFIX}(?P<command>файл ид)(?:$|\n)")).unwrap())
}

#[inline]
pub fn re_minimal_rate() -> &'static Regex {
    RE_MINIMAL_RATE.get_or_init(|| {
        Regex::new(&format!(r"(?i)^{PREFIX}(?P<command>мин ставка)(?:$|\n)")).unwrap()
    })
}

// CALLBACK REGEX
#[inline]
pub fn re_callback_captcha() -> &'static Regex {
    RE_CALLBACK_CAPTCHA.get_or_init(|| {
        Regex::new(r"^captcha:(?P<chat_id>-?\d+):(?P<user_id>\d+):(?P<code>\d+)").unwrap()
    })
}

#[inline]
pub fn re_del_sum() -> &'static Regex {
    RE_DEL_SUM.get_or_init(|| Regex::new(r"^del_sum:(?P<summon_id>[a-f0-9]{32})").unwrap())
}

#[inline]
pub fn re_repeat_reg() -> &'static Regex {
    RE_REPEAT_REG
        .get_or_init(|| Regex::new(r"^repeat_reg:(?P<chat_id>-?\d+):(?P<user_id>\d+)$").unwrap())
}

#[inline]
pub fn re_unmute() -> &'static Regex {
    RE_UNMUTE.get_or_init(|| {
        Regex::new(r"^unmute:(?P<chat_id>-?\d+):(?P<message_id>\d+|none):(?P<user_id>\d+)$")
            .unwrap()
    })
}

#[inline]
pub fn re_ban() -> &'static Regex {
    RE_BAN.get_or_init(|| {
        Regex::new(r"^ban:(?P<chat_id>-?\d+):(?P<message_id>\d+|none):(?P<user_id>\d+)$").unwrap()
    })
}
