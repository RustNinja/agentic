pub struct DeadStrSplitInclusiveRevMapItem {
    value: String,
}

impl DeadStrSplitInclusiveRevMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-split-inclusive-rev-map:{}", self.value)
    }
}

pub fn dead_str_split_inclusive_rev_map(raw: &str) -> String {
    DeadStrSplitInclusiveRevMapItem::new(raw).dead_method()
}
