pub struct DeadStrRmatchIndicesMapItem {
    value: String,
}

impl DeadStrRmatchIndicesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-rmatch-indices-map:{}", self.value)
    }
}

pub fn dead_str_rmatch_indices_map(raw: &str) -> String {
    DeadStrRmatchIndicesMapItem::new(raw).dead_method()
}
