pub struct DeadPathbufReservePushMapItem {
    value: String,
}

impl DeadPathbufReservePushMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-reserve-push-map:{}", self.value)
    }
}

pub fn dead_pathbuf_reserve_push_map(raw: &str) -> String {
    DeadPathbufReservePushMapItem::new(raw).dead_method()
}
