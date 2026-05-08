pub struct TryStep {
    value: String,
}

impl TryStep {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn append_to(&self, output: &mut Vec<String>) -> Result<(), TryError> {
        if self.value == "fail" {
            Err(TryError::new(&self.value))
        } else {
            output.push(format!("try:{}", self.value));
            Ok(())
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-try-step:{}", self.value)
    }
}

pub struct TryError {
    label: String,
}

impl TryError {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("try-error:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-try-error:{}", self.label)
    }
}

fn build_steps(raw: &str) -> Vec<TryStep> {
    raw.split(',').map(TryStep::new).collect()
}

pub fn selected_try_for_each(raw: &str) -> String {
    let mut rendered = Vec::new();
    build_steps(raw)
        .iter()
        .try_for_each(|step| step.append_to(&mut rendered))
        .map(|_| rendered.join("|"))
        .unwrap_or_else(|err| err.render())
}

pub fn dead_live_try_for_each(raw: &str) -> String {
    TryStep::new(raw).dead_method()
}
