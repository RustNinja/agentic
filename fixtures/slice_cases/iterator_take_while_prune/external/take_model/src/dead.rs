pub struct DeadTakeItem {
    value: String,
}

impl DeadTakeItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-take:{}", self.value)
    }
}

pub fn dead_take_while(raw: &str) -> String {
    DeadTakeItem::new(raw).render()
}
