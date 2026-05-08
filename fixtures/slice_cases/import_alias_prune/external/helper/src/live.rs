pub fn normalize_value(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub fn format_label(value: &str) -> String {
    format!("helper:{value}")
}

pub fn unused_helper(value: &str) -> String {
    format!("unused-helper:{value}")
}
