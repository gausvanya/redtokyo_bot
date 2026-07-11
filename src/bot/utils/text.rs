pub fn clear_text(text: String) -> String {
    let mut result = String::new();
    let mut last_was_separator = false;

    for c in text.chars() {
        if c.is_alphanumeric() {
            result.push(c);
            last_was_separator = false;
        } else {
            if !result.is_empty() && !last_was_separator {
                result.push(' ');
                last_was_separator = true;
            }
        }
    }

    if result.ends_with(' ') {
        result.pop();
    }

    result
}
