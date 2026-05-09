pub struct DeadStrRsplitnFindMapItem {
    value: String,
}

impl DeadStrRsplitnFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-rsplitn-find-map:{}", self.value)
    }
}

pub fn dead_str_rsplitn_find_map(raw: &str) -> String {
    DeadStrRsplitnFindMapItem::new(raw).dead_method()
}
