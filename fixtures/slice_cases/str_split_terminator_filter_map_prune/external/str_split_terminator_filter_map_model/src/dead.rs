pub struct DeadStrSplitTerminatorFilterMapItem {
    value: String,
}

impl DeadStrSplitTerminatorFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-split-terminator-filter-map:{}", self.value)
    }
}

pub fn dead_str_split_terminator_filter_map(raw: &str) -> String {
    DeadStrSplitTerminatorFilterMapItem::new(raw).dead_method()
}
