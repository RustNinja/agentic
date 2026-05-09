pub struct DeadHashsetIntersectionItem {
    value: String,
}

impl DeadHashsetIntersectionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-hashset-intersection:{}", self.value)
    }
}

pub fn dead_hashset_intersection(raw: &str) -> String {
    DeadHashsetIntersectionItem::new(raw).render()
}
