pub struct WhileStep {
    value: String,
}

impl WhileStep {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn next_render(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| format!("while:{}", self.value))
    }

    pub fn dead_method(&self) -> String {
        format!("dead-while:{}", self.value)
    }
}

fn build_steps(raw: &str) -> Vec<WhileStep> {
    raw.split(',').map(WhileStep::new).collect()
}

pub fn selected_map_while(raw: &str) -> String {
    build_steps(raw)
        .iter()
        .map_while(|step| step.next_render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_map_while(raw: &str) -> String {
    WhileStep::new(raw).dead_method()
}
