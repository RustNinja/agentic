pub struct DeadRwlockGetMutMapItem {
    value: String,
}

impl DeadRwlockGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-get-mut-map:{}", self.value)
    }
}

pub fn dead_rwlock_get_mut_map(raw: &str) -> String {
    DeadRwlockGetMutMapItem::new(raw).dead_method()
}
