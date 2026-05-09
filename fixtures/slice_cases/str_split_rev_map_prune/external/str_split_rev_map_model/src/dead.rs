pub struct DeadStrSplitRevMapItem {
    value: String,
}

impl DeadStrSplitRevMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-split-rev-map:{}", self.value)
    }
}

pub fn dead_str_split_rev_map(raw: &str) -> String {
    DeadStrSplitRevMapItem::new(raw).dead_method()
}
