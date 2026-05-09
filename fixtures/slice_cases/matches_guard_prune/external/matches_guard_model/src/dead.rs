pub struct DeadMatchesGuardItem {
    value: String,
}

impl DeadMatchesGuardItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-matches-guard:{}", self.value)
    }
}

pub fn dead_matches_guard(raw: &str) -> String {
    DeadMatchesGuardItem::new(raw).render()
}
