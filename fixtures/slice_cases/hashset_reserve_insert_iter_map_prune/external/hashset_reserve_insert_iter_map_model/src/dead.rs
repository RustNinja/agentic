pub struct DeadHashsetReserveInsertIterMapItem {
    value: String,
}

impl DeadHashsetReserveInsertIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-reserve-insert-iter-map:{}", self.value)
    }
}

pub fn dead_hashset_reserve_insert_iter_map(raw: &str) -> String {
    DeadHashsetReserveInsertIterMapItem::new(raw).dead_method()
}
