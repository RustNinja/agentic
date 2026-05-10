pub struct DeadStrTrimStartMatchesMapItem {
    value: String,
}

impl DeadStrTrimStartMatchesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-trim-start-matches-map:{}", self.value)
    }
}

pub fn dead_str_trim_start_matches_map(raw: &str) -> String {
    DeadStrTrimStartMatchesMapItem::new(raw).dead_method()
}
