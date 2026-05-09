pub struct DeadOptionOkOrMapItem {
    value: String,
}

impl DeadOptionOkOrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-ok-or-map:{}", self.value)
    }
}

pub fn dead_option_ok_or_map(raw: &str) -> String {
    DeadOptionOkOrMapItem::new(raw).render()
}
