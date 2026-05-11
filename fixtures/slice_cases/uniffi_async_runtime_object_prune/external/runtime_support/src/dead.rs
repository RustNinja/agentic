pub struct DeadRuntime {
    raw: String,
}

impl DeadRuntime {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub async fn render(self) -> String {
        format!("dead-runtime:{}", self.raw)
    }
}

pub async fn dead_runtime_report(raw: &str) -> String {
    DeadRuntime::new(raw).render().await
}
