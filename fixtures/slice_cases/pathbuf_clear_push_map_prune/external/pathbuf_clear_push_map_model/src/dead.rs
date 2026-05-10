pub struct DeadPathbufClearPushMapItem {
    value: String,
}

impl DeadPathbufClearPushMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-clear-push-map:{}", self.value)
    }
}

pub fn dead_pathbuf_clear_push_map(raw: &str) -> String {
    DeadPathbufClearPushMapItem::new(raw).dead_method()
}
