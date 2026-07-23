use regex::Regex;
use std::sync::OnceLock;

const PREFIX: &str = r#"(?:[.!/]\s?|сап\s?)?"#;
const USER_PATTERN: &str = r#"(?:\s+(?:https?://t\.me/|@|tg://(?:user\?id=|openmessage\?user_id=|resolve\?domain=)|<a\s+href=["']tg://user\?id=)?(?P<user>-\d+|\d+|[a-zA-Z0-9_]{5,32})(?:["']?>[^<]*</a>)?)?"#;

macro_rules! regex {
    ($lock:expr, $pattern:expr) => {
        $lock.get_or_init(|| Regex::new($pattern).expect("Failed to compile regex"))
    };
}

// COMMANDS REGEX STATIC
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
static DB_UPDATE_RATE: OnceLock<Regex> = OnceLock::new();

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
    regex!(
        RE_PING,
        &format!(r"(?i)^{PREFIX}(?P<command>пинг|ping)(?:$|\n)")
    )
}

#[inline]
pub fn re_duel() -> &'static Regex {
    regex!(
        RE_DUEL,
        r"(?i)^(?:[!./]|ириска?|ирис\s+)?(?P<command>кто дуэль|кто кубы|дуэль|кубы)\s*(?P<amount>\d+[кk]?)?(?:$|\n)"
    )
}

#[inline]
pub fn re_set_garant() -> &'static Regex {
    regex!(
        RE_SET_GARANT,
        &format!(r#"(?i)^{PREFIX}\+гарант{USER_PATTERN}\s*\n(?P<comment>[\s\S]+)"#)
    )
}

#[inline]
pub fn re_remove_garant() -> &'static Regex {
    regex!(
        RE_REMOVE_GARANT,
        &format!(r#"(?i)^{PREFIX}-гарант{USER_PATTERN}\s*(?:$|\n)"#)
    )
}

#[inline]
pub fn re_call_garants() -> &'static Regex {
    regex!(
        RE_CALL_GARANTS,
        &format!(r#"(?i)^{PREFIX}созвать\s+гарантов(?:\s*\n(?P<reason>[\s\S]+))?"#)
    )
}

#[inline]
pub fn re_list_garants() -> &'static Regex {
    regex!(
        RE_LIST_GARANTS,
        &format!(r#"(?i)^{PREFIX}(?:кто\s+)?гаранты(?:$|\n)"#)
    )
}

#[inline]
pub fn re_set_warn() -> &'static Regex {
    regex!(
        RE_SET_WARN,
        &format!(r#"(?i)^{PREFIX}\+уст{USER_PATTERN}\s*\n(?P<reason>[\s\S]+)"#)
    )
}

#[inline]
pub fn re_remove_warn() -> &'static Regex {
    RE_REMOVE_WARN.get_or_init(|| {
        Regex::new(&format!(r#"(?i)^{PREFIX}-уст{USER_PATTERN}\s*(?:$|\n)"#)).unwrap()
    })
}

#[inline]
pub fn re_list_warns() -> &'static Regex {
    regex!(
        RE_LIST_WARNS,
        &format!(r#"(?i)^{PREFIX}(?P<command>мои\s+усты|твои\s+усты){USER_PATTERN}\s*(?:$|\n)"#)
    )
}

#[inline]
pub fn re_set_scam() -> &'static Regex {
    regex!(
        RE_SET_SCAM,
        &format!(r#"(?i)^{PREFIX}\+скам база{USER_PATTERN}?\s*\n(?P<reason>[\s\S]+)"#)
    )
}

#[inline]
pub fn re_remove_scam() -> &'static Regex {
    regex!(
        RE_REMOVE_SCAM,
        &format!(r#"(?P<command>(?i)^{PREFIX}-скам база(?:\s+ошибка)?){USER_PATTERN}(?:$|\s+)"#)
    )
}

#[inline]
pub fn re_reason_scam() -> &'static Regex {
    regex!(
        RE_REASON_SCAM,
        &format!(r#"(?i)^{PREFIX}причина{USER_PATTERN}(?:$|\s+)"#)
    )
}

#[inline]
pub fn re_file_id() -> &'static Regex {
    regex!(
        RE_FILE_ID,
        &format!(r"(?i)^{PREFIX}(?P<command>файл ид)(?:$|\n)")
    )
}

#[inline]
pub fn re_minimal_rate() -> &'static Regex {
    regex!(
        RE_MINIMAL_RATE,
        &format!(r"(?i)^{PREFIX}(?P<command>мин ставка)(?:$|\n)")
    )
}

#[inline]
pub fn re_db_update() -> &'static Regex {
    DB_UPDATE_RATE.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)^{PREFIX}обновить бд{USER_PATTERN}(?:$|\s+)"#
        ))
        .unwrap()
    })
}

// CALLBACK REGEX
#[inline]
pub fn re_callback_captcha() -> &'static Regex {
    regex!(
        RE_CALLBACK_CAPTCHA,
        r"^captcha:(?P<chat_id>-?\d+):(?P<user_id>\d+):(?P<code>\d+)"
    )
}

#[inline]
pub fn re_del_sum() -> &'static Regex {
    regex!(
        RE_DEL_SUM,
        r"^del_sum:(?P<summon_id>[a-f0-9]{32})"
    )
}

#[inline]
pub fn re_repeat_reg() -> &'static Regex {
    regex!(
        RE_REPEAT_REG,
        r"^repeat_reg:(?P<chat_id>-?\d+):(?P<user_id>\d+)$"
    )
}

#[inline]
pub fn re_unmute() -> &'static Regex {
    regex!(
        RE_UNMUTE,
        r"^unmute:(?P<chat_id>-?\d+):(?P<message_id>\d+|none):(?P<user_id>\d+)$"
    )
}

#[inline]
pub fn re_ban() -> &'static Regex {
    regex!(
        RE_BAN,
        r"^ban:(?P<chat_id>-?\d+):(?P<message_id>\d+|none):(?P<user_id>\d+)$"
    )
}
