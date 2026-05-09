pub struct DeadOnceLockGetOrInitMapItem {
    value: String,
}

impl DeadOnceLockGetOrInitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-once-lock-get-or-init-map:{}", self.value)
    }
}

pub fn dead_once_lock_get_or_init_map(raw: &str) -> String {
    DeadOnceLockGetOrInitMapItem::new(raw).render()
}
