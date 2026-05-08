pub struct ClosureReturnRecord {
    label: String,
}

impl ClosureReturnRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("closure-return:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-closure-return:{}", self.label)
    }
}

pub fn selected_closure_return(raw: &str) -> String {
    let build = || ClosureReturnRecord::new(raw);
    build().render()
}

pub fn dead_live_closure_return(raw: &str) -> String {
    ClosureReturnRecord::new(raw).dead_method()
}
