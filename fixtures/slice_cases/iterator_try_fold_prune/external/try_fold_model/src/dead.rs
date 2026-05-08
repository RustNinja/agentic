pub struct DeadTryFoldItem {
    value: String,
}

impl DeadTryFoldItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-try-fold:{}", self.value)
    }
}

pub fn dead_try_fold(raw: &str) -> String {
    DeadTryFoldItem::new(raw).render()
}
