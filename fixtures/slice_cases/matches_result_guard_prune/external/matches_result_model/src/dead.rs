pub struct DeadMatchesResultItem {
    value: String,
}

impl DeadMatchesResultItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-matches-result:{}", self.value)
    }
}

pub fn dead_matches_result(raw: &str) -> String {
    DeadMatchesResultItem::new(raw).render()
}
