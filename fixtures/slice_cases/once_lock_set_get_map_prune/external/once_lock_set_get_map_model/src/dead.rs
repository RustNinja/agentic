pub struct DeadOnceLockSetGetMapItem {
    value: String,
}

impl DeadOnceLockSetGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-set-get-map:{}", self.value)
    }
}

pub fn dead_once_lock_set_get_map(raw: &str) -> String {
    DeadOnceLockSetGetMapItem::new(raw).dead_method()
}
