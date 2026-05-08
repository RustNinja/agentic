pub struct DeadRuntimeObject {
    prefix: String,
}

impl DeadRuntimeObject {
    pub fn new() -> Self {
        Self {
            prefix: "dead".to_string(),
        }
    }

    pub fn status(self, raw: &str) -> String {
        format!("{}:{raw}", self.prefix)
    }
}

pub fn dead_runtime(raw: &str) -> String {
    DeadRuntimeObject::new().status(raw)
}
