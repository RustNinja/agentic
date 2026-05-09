pub struct DeadStrMatchesMapItem {
    value: String,
}

impl DeadStrMatchesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-matches-map:{}", self.value)
    }
}

pub fn dead_str_matches_map(raw: &str) -> String {
    DeadStrMatchesMapItem::new(raw).dead_method()
}
