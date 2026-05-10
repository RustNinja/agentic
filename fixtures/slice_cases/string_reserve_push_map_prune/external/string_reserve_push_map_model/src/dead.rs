pub struct DeadStringReservePushMapItem {
    value: String,
}

impl DeadStringReservePushMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-reserve-push-map:{}", self.value)
    }
}

pub fn dead_string_reserve_push_map(raw: &str) -> String {
    DeadStringReservePushMapItem::new(raw).dead_method()
}
