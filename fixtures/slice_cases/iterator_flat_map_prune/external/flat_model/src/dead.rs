pub struct DeadFlatSegment {
    value: String,
}

impl DeadFlatSegment {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-flat-segment:{}", self.value)
    }
}

pub fn dead_flat_map(raw: &str) -> String {
    DeadFlatSegment::new(raw).render()
}
