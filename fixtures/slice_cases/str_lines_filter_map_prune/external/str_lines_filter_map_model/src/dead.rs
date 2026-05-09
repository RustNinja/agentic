pub struct DeadStrLinesFilterMapItem {
    value: String,
}

impl DeadStrLinesFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-lines-filter-map:{}", self.value)
    }
}

pub fn dead_str_lines_filter_map(raw: &str) -> String {
    DeadStrLinesFilterMapItem::new(raw).dead_method()
}
