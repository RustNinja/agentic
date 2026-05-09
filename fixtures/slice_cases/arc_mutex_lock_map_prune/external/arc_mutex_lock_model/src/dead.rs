pub struct DeadArcMutexLockMapItem {
    value: String,
}

impl DeadArcMutexLockMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-arc-mutex-lock-map:{}", self.value)
    }
}

pub fn dead_arc_mutex_lock_map(raw: &str) -> String {
    DeadArcMutexLockMapItem::new(raw).render()
}
