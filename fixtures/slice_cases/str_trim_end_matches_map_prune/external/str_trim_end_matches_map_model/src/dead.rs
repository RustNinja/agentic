pub struct DeadStrTrimEndMatchesMapItem {
    value: String,
}

impl DeadStrTrimEndMatchesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-trim-end-matches-map:{}", self.value)
    }
}

pub fn dead_str_trim_end_matches_map(raw: &str) -> String {
    DeadStrTrimEndMatchesMapItem::new(raw).dead_method()
}
