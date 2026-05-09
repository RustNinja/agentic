pub struct DeadStrTrimMatchesMapItem {
    value: String,
}

impl DeadStrTrimMatchesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-trim-matches-map:{}", self.value)
    }
}

pub fn dead_str_trim_matches_map(raw: &str) -> String {
    DeadStrTrimMatchesMapItem::new(raw).dead_method()
}
