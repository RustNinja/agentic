pub struct DeadMutexLockMutMapItem {
    value: String,
}

impl DeadMutexLockMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-mutex-lock-mut-map:{}", self.value)
    }
}

pub fn dead_mutex_lock_mut_map(raw: &str) -> String {
    DeadMutexLockMutMapItem::new(raw).render()
}
