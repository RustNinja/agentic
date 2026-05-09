pub struct DeadOptionIterClonedMapItem {
    value: String,
}

impl DeadOptionIterClonedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-iter-cloned-map:{}", self.value)
    }
}

pub fn dead_option_iter_cloned_map(raw: &str) -> String {
    DeadOptionIterClonedMapItem::new(raw).render()
}
