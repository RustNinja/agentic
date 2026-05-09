pub struct DeadOptionIterMapItem {
    value: String,
}

impl DeadOptionIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-iter-map:{}", self.value)
    }
}

pub fn dead_option_iter_map(raw: &str) -> String {
    DeadOptionIterMapItem::new(raw).render()
}
