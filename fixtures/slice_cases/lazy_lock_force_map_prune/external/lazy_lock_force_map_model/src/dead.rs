pub struct DeadLazyLockForceMapItem {
    value: String,
}

impl DeadLazyLockForceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-lazy-lock-force-map:{}", self.value)
    }
}

pub fn dead_lazy_lock_force_map(raw: &str) -> String {
    DeadLazyLockForceMapItem::new(raw).dead_method()
}
