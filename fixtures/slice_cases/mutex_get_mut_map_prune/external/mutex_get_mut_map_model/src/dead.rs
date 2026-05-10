pub struct DeadMutexGetMutMapItem {
    value: String,
}

impl DeadMutexGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-get-mut-map:{}", self.value)
    }
}

pub fn dead_mutex_get_mut_map(raw: &str) -> String {
    DeadMutexGetMutMapItem::new(raw).dead_method()
}
