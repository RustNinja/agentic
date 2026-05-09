pub struct DeadRwLockReadMapItem {
    value: String,
}

impl DeadRwLockReadMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-rwlock-read-map:{}", self.value)
    }
}

pub fn dead_rwlock_read_map(raw: &str) -> String {
    DeadRwLockReadMapItem::new(raw).render()
}
