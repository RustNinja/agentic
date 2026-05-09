pub struct DeadResultLetItem {
    value: String,
}

impl DeadResultLetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-let:{}", self.value)
    }
}

pub fn dead_result_let(raw: &str) -> String {
    DeadResultLetItem::new(raw).render()
}
