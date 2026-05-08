pub struct DeadEnumFlatMapItem {
    value: String,
}

impl DeadEnumFlatMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-enum-flat-map:{}", self.value)
    }
}

pub fn dead_enum_flat_map(raw: &str) -> String {
    DeadEnumFlatMapItem::new(raw).render()
}
