pub struct DeadCowToMutMapItem {
    value: String,
}

impl DeadCowToMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-cow-to-mut-map:{}", self.value)
    }
}

pub fn dead_cow_to_mut_map(raw: &str) -> String {
    DeadCowToMutMapItem::new(raw).render()
}
