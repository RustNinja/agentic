pub struct DeadPathbufPopMapItem {
    value: String,
}

impl DeadPathbufPopMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-pop-map:{}", self.value)
    }
}

pub fn dead_pathbuf_pop_map(raw: &str) -> String {
    DeadPathbufPopMapItem::new(raw).dead_method()
}
