pub struct DeadStrSplitWhitespaceFindMapItem {
    value: String,
}

impl DeadStrSplitWhitespaceFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-split-whitespace-find-map:{}", self.value)
    }
}

pub fn dead_str_split_whitespace_find_map(raw: &str) -> String {
    DeadStrSplitWhitespaceFindMapItem::new(raw).dead_method()
}
