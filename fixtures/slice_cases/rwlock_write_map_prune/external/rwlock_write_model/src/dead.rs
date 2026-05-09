pub struct DeadRwLockWriteMapItem {
    value: String,
}

impl DeadRwLockWriteMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-rwlock-write-map:{}", self.value)
    }
}

pub fn dead_rwlock_write_map(raw: &str) -> String {
    DeadRwLockWriteMapItem::new(raw).render()
}
