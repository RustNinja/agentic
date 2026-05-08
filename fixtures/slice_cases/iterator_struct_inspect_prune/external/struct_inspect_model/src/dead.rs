pub struct DeadStructInspectItem {
    value: String,
}

impl DeadStructInspectItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-struct-inspect:{}", self.value)
    }
}

pub fn dead_struct_inspect(raw: &str) -> String {
    DeadStructInspectItem::new(raw).render()
}
