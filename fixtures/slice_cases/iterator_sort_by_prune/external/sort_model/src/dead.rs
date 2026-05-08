pub struct DeadSortItem {
    value: String,
}

impl DeadSortItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-sort:{}", self.value)
    }
}

pub fn dead_sort_by(raw: &str) -> String {
    DeadSortItem::new(raw).render()
}
