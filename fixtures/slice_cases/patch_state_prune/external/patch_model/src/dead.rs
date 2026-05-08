pub struct DeadPatch {
    raw: String,
}

impl DeadPatch {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-patch:{}", self.raw)
    }
}

pub fn dead_patch(raw: &str) -> String {
    DeadPatch::new(raw).render()
}
