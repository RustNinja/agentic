pub struct DeadWhileLetItem {
    value: String,
}

impl DeadWhileLetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-while-let:{}", self.value)
    }
}

pub fn dead_while_let(raw: &str) -> String {
    DeadWhileLetItem::new(raw).render()
}
