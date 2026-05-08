pub struct DeadTupleSortKeyItem {
    value: String,
}

impl DeadTupleSortKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-sort-by-key:{}", self.value)
    }
}

pub fn dead_tuple_sort_by_key(raw: &str) -> String {
    DeadTupleSortKeyItem::new(raw).render()
}
