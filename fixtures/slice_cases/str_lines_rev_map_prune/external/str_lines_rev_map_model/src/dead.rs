pub struct DeadStrLinesRevMapItem {
    value: String,
}

impl DeadStrLinesRevMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-lines-rev-map:{}", self.value)
    }
}

pub fn dead_str_lines_rev_map(raw: &str) -> String {
    DeadStrLinesRevMapItem::new(raw).dead_method()
}
