pub struct DeadPatchRequest {
    raw: String,
}

impl DeadPatchRequest {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead:{}", self.raw)
    }
}

pub fn dead_patch_summary(raw: &str) -> String {
    DeadPatchRequest::new(raw).render()
}
