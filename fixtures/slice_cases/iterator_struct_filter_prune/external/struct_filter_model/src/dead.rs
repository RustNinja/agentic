pub struct DeadStructFilterItem {
    value: String,
}

impl DeadStructFilterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-struct-filter:{}", self.value)
    }
}

pub fn dead_struct_filter(raw: &str) -> String {
    DeadStructFilterItem::new(raw).render()
}
