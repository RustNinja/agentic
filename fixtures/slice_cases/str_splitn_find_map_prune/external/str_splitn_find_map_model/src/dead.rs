pub struct DeadStrSplitnFindMapItem {
    value: String,
}

impl DeadStrSplitnFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-splitn-find-map:{}", self.value)
    }
}

pub fn dead_str_splitn_find_map(raw: &str) -> String {
    DeadStrSplitnFindMapItem::new(raw).dead_method()
}
