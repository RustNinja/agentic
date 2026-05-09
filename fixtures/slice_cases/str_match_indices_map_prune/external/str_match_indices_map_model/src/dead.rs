pub struct DeadStrMatchIndicesMapItem {
    value: String,
}

impl DeadStrMatchIndicesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-match-indices-map:{}", self.value)
    }
}

pub fn dead_str_match_indices_map(raw: &str) -> String {
    DeadStrMatchIndicesMapItem::new(raw).dead_method()
}
