pub struct DeadStructMapItem {
    value: String,
}

impl DeadStructMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-struct-map:{}", self.value)
    }
}

pub fn dead_struct_map(raw: &str) -> String {
    DeadStructMapItem::new(raw).render()
}
