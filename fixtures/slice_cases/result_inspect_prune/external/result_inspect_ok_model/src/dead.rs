pub struct DeadResultInspectOkItem {
    value: String,
}

impl DeadResultInspectOkItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-inspect-ok:{}", self.value)
    }
}

pub fn dead_result_inspect_ok(raw: &str) -> String {
    DeadResultInspectOkItem::new(raw).render()
}
