pub struct DeadBtreesetIntersectionItem {
    value: String,
}

impl DeadBtreesetIntersectionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreeset-intersection:{}", self.value)
    }
}

pub fn dead_btreeset_intersection(raw: &str) -> String {
    DeadBtreesetIntersectionItem::new(raw).render()
}
