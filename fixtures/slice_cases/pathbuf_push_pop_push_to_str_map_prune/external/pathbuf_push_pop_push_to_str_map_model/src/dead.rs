pub struct DeadPathbufPushPopPushToStrMapItem {
    value: String,
}

impl DeadPathbufPushPopPushToStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-push-pop-push-to-str-map:{}", self.value)
    }
}

pub fn dead_pathbuf_push_pop_push_to_str_map(raw: &str) -> String {
    DeadPathbufPushPopPushToStrMapItem::new(raw).dead_method()
}
