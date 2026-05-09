pub struct DeadOnceLockGetMapItem {
    value: String,
}

impl DeadOnceLockGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-once-lock-get-map:{}", self.value)
    }
}

pub fn dead_once_lock_get_map(raw: &str) -> String {
    DeadOnceLockGetMapItem::new(raw).render()
}
