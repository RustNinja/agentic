pub struct DeadIteratorFlatMapMatchEnumVecItem {
    value: String,
}

impl DeadIteratorFlatMapMatchEnumVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flat-map-match-enum-vec:{}", self.value)
    }
}

pub fn dead_iterator_flat_map_match_enum_vec(raw: &str) -> String {
    DeadIteratorFlatMapMatchEnumVecItem::new(raw).dead_method()
}
