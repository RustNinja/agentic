pub struct DeadOptionIntoIterNextMapItem {
    value: String,
}

impl DeadOptionIntoIterNextMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-into-iter-next-map:{}", self.value)
    }
}

pub fn dead_option_into_iter_next_map(raw: &str) -> String {
    DeadOptionIntoIterNextMapItem::new(raw).dead_method()
}
