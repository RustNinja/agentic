pub struct DeadHashmapIterFilterMapMatchItem {
    value: String,
}

impl DeadHashmapIterFilterMapMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-filter-map-match:{}", self.value)
    }
}

pub fn dead_hashmap_iter_filter_map_match(raw: &str) -> String {
    DeadHashmapIterFilterMapMatchItem::new(raw).dead_method()
}
