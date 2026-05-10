pub struct DeadArcGetMutMapItem {
    value: String,
}

impl DeadArcGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-get-mut-map:{}", self.value)
    }
}

pub fn dead_arc_get_mut_map(raw: &str) -> String {
    DeadArcGetMutMapItem::new(raw).dead_method()
}
