pub struct DeadMutexLockMapItem {
    value: String,
}

impl DeadMutexLockMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-mutex-lock-map:{}", self.value)
    }
}

pub fn dead_mutex_lock_map(raw: &str) -> String {
    DeadMutexLockMapItem::new(raw).render()
}
