use std::sync::OnceLock;

use regex::Regex;

const PREFIX: &str = r#"(?:[.!/]\s?|сап\s?)?"#;
const USER_PATTERN: &str = r#"(?:https?://(?:t\.me|telegram\.(?:org|dog))/|@|tg://(?:user\?id=|openmessage\?user_id=|resolve\?domain=)|<a\s+href=["']tg://user\?id=)?(?P<user>-\d+|\d+|[a-zA-Z0-9_]{5,32})(?:["']?>[^<]*</a>)?"#;

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
pub static RE_REFERRAL: OnceLock<Regex> = OnceLock::new();

// COMMANDS REGEX
#[inline]
pub fn re_ping() -> &'static Regex {
    regex!(RE_PING, &format!(r#"(?i)^{PREFIX}(?P<command>пинг|ping)(?:\n[\s\S]*)?$"#))
}

#[inline]
pub fn re_duel() -> &'static Regex {
    regex!(
        RE_DUEL,
        r#"(?i)^(?:[!./]|ириска?|ирис\s+)?(?P<command>кто дуэль|кто кубы|дуэль|кубы)(?:\s+(?P<amount>\d+[кk]?))?(?:\n[\s\S]*)?$"#
    )
}

#[inline]
pub fn re_set_garant() -> &'static Regex {
    regex!(
        RE_SET_GARANT,
        &format!(r#"(?i)^{PREFIX}\+гарант(?:\s+{USER_PATTERN})?\s*\n(?P<comment>[\s\S]+)$"#)
    )
}

#[inline]
pub fn re_remove_garant() -> &'static Regex {
    regex!(
        RE_REMOVE_GARANT,
        &format!(r#"(?i)^{PREFIX}-гарант(?:\s+{USER_PATTERN})?(?:\n[\s\S]*)?$"#)
    )
}

#[inline]
pub fn re_call_garants() -> &'static Regex {
    regex!(
        RE_CALL_GARANTS,
        &format!(r#"(?i)^{PREFIX}созвать\s+гарантов(?:\s*\n(?P<reason>[\s\S]+))?$"#)
    )
}

#[inline]
pub fn re_list_garants() -> &'static Regex {
    regex!(RE_LIST_GARANTS, &format!(r#"(?i)^{PREFIX}(?:кто\s+)?гаранты(?:\n[\s\S]*)?$"#))
}

#[inline]
pub fn re_set_warn() -> &'static Regex {
    regex!(
        RE_SET_WARN,
        &format!(r#"(?i)^{PREFIX}\+уст(?:\s+{USER_PATTERN})?\s*\n(?P<reason>[\s\S]+)$"#)
    )
}

#[inline]
pub fn re_remove_warn() -> &'static Regex {
    regex!(RE_REMOVE_WARN, &format!(r#"(?i)^{PREFIX}-уст(?:\s+{USER_PATTERN})?(?:\n[\s\S]*)?$"#))
}

#[inline]
pub fn re_list_warns() -> &'static Regex {
    regex!(
        RE_LIST_WARNS,
        &format!(
            r#"(?i)^{PREFIX}(?P<command>мои\s+усты|твои\s+усты)(?:\s+{USER_PATTERN})?(?:\n[\s\S]*)?$"#
        )
    )
}

#[inline]
pub fn re_set_scam() -> &'static Regex {
    regex!(
        RE_SET_SCAM,
        &format!(r#"(?i)^{PREFIX}\+скам база(?:\s+{USER_PATTERN})?\s*\n(?P<reason>[\s\S]+)$"#)
    )
}

#[inline]
pub fn re_remove_scam() -> &'static Regex {
    regex!(
        RE_REMOVE_SCAM,
        &format!(
            r#"(?i)^{PREFIX}(?P<command>-скам база(?:\s+ошибка)?)(?:\s+{USER_PATTERN})?(?:\n[\s\S]*)?$"#
        )
    )
}

#[inline]
pub fn re_reason_scam() -> &'static Regex {
    regex!(RE_REASON_SCAM, &format!(r#"(?i)^{PREFIX}причина(?:\s+{USER_PATTERN})?(?:\n[\s\S]*)?$"#))
}

#[inline]
pub fn re_file_id() -> &'static Regex {
    regex!(RE_FILE_ID, &format!(r#"(?i)^{PREFIX}(?P<command>файл ид)(?:\n[\s\S]*)?$"#))
}

#[inline]
pub fn re_minimal_rate() -> &'static Regex {
    regex!(RE_MINIMAL_RATE, &format!(r#"(?i)^{PREFIX}(?P<command>мин ставка)(?:\n[\s\S]*)?$"#))
}

#[inline]
pub fn re_db_update() -> &'static Regex {
    regex!(
        DB_UPDATE_RATE,
        &format!(r#"(?i)^{PREFIX}обновить бд(?:\s+{USER_PATTERN})?(?:\n[\s\S]*)?$"#)
    )
}

// CALLBACK REGEX
#[inline]
pub fn re_callback_captcha() -> &'static Regex {
    regex!(RE_CALLBACK_CAPTCHA, r"^captcha:(?P<chat_id>-?\d+):(?P<user_id>\d+):(?P<code>\d+)$")
}

#[inline]
pub fn re_del_sum() -> &'static Regex {
    regex!(RE_DEL_SUM, r"^del_sum:(?P<summon_id>[a-f0-9]{32})$")
}

#[inline]
pub fn re_repeat_reg() -> &'static Regex {
    regex!(RE_REPEAT_REG, r"^repeat_reg:(?P<chat_id>-?\d+):(?P<user_id>\d+)$")
}

#[inline]
pub fn re_unmute() -> &'static Regex {
    regex!(RE_UNMUTE, r"^unmute:(?P<chat_id>-?\d+):(?P<message_id>\d+|none):(?P<user_id>\d+)$")
}

#[inline]
pub fn re_ban() -> &'static Regex {
    regex!(RE_BAN, r"^ban:(?P<chat_id>-?\d+):(?P<message_id>\d+|none):(?P<user_id>\d+)$")
}

#[inline]
pub fn re_referral() -> &'static Regex {
    regex!(RE_REFERRAL, r"(?:t\.me|telegram\.(?:org|me|dog))/(?:\+\w+|gram_piarbot\?start=check_)")
}
