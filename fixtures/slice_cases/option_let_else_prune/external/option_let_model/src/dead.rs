pub struct DeadOptionLetItem {
    value: String,
}

impl DeadOptionLetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-let:{}", self.value)
    }
}

pub fn dead_option_let(raw: &str) -> String {
    DeadOptionLetItem::new(raw).render()
}
