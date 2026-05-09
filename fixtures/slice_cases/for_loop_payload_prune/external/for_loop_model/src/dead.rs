pub struct DeadForLoopItem {
    value: String,
}

impl DeadForLoopItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-for-loop:{}", self.value)
    }
}

pub fn dead_for_loop(raw: &str) -> String {
    DeadForLoopItem::new(raw).render()
}
