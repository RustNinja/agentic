pub struct DeadHashmapReserveInsertValuesMapItem {
    value: String,
}

impl DeadHashmapReserveInsertValuesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-reserve-insert-values-map:{}", self.value)
    }
}

pub fn dead_hashmap_reserve_insert_values_map(raw: &str) -> String {
    DeadHashmapReserveInsertValuesMapItem::new(raw).dead_method()
}
