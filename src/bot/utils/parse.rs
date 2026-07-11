pub fn parse_amount(input: &str) -> i64 {
    let input = input.trim().to_lowercase();

    if input.starts_with("http") || input.contains("t.me") {
        return -1;
    }

    if let Some(stripped) = input.strip_suffix('k')
        && let Ok(v) = stripped.parse::<i64>()
    {
        return v * 1000;
    }

    if let Some(stripped) = input.strip_suffix('к')
        && let Ok(v) = stripped.parse::<i64>()
    {
        return v * 1000;
    }

    if let Ok(v) = input.parse::<i64>() {
        return v;
    }

    0
}
