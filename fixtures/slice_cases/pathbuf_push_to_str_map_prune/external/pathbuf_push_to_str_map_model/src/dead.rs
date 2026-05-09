pub struct DeadPathbufPushToStrMapItem {
    value: String,
}

impl DeadPathbufPushToStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-push-to-str-map:{}", self.value)
    }
}

pub fn dead_pathbuf_push_to_str_map(raw: &str) -> String {
    DeadPathbufPushToStrMapItem::new(raw).dead_method()
}
