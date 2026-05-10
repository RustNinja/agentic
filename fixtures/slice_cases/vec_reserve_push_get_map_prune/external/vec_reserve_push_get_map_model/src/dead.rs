pub struct DeadVecReservePushGetMapItem {
    value: String,
}

impl DeadVecReservePushGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-reserve-push-get-map:{}", self.value)
    }
}

pub fn dead_vec_reserve_push_get_map(raw: &str) -> String {
    DeadVecReservePushGetMapItem::new(raw).dead_method()
}
