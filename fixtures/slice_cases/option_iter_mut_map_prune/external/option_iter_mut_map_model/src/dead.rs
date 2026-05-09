pub struct DeadOptionIterMutMapItem {
    value: String,
}

impl DeadOptionIterMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-iter-mut-map:{}", self.value)
    }
}

pub fn dead_option_iter_mut_map(raw: &str) -> String {
    DeadOptionIterMutMapItem::new(raw).render()
}
