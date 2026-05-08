pub struct TryFoldStep {
    value: String,
}

impl TryFoldStep {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn append_to(&self, mut state: TryFoldState) -> Result<TryFoldState, TryFoldError> {
        if self.value == "fail" {
            Err(TryFoldError::new(&self.value))
        } else {
            state.parts.push(format!("try-fold:{}", self.value));
            Ok(state)
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-try-fold-step:{}", self.value)
    }
}

pub struct TryFoldState {
    parts: Vec<String>,
}

impl TryFoldState {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn render(self) -> String {
        self.parts.join("|")
    }

    pub fn dead_method(self) -> String {
        format!("dead-try-fold-state:{}", self.parts.len())
    }
}

pub struct TryFoldError {
    label: String,
}

impl TryFoldError {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("try-fold-error:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-try-fold-error:{}", self.label)
    }
}

fn build_steps(raw: &str) -> Vec<TryFoldStep> {
    raw.split(',').map(TryFoldStep::new).collect()
}

pub fn selected_try_fold(raw: &str) -> String {
    build_steps(raw)
        .iter()
        .try_fold(TryFoldState::new(), |state, step| step.append_to(state))
        .map(|state| state.render())
        .unwrap_or_else(|err| err.render())
}

pub fn dead_live_try_fold(raw: &str) -> String {
    TryFoldStep::new(raw).dead_method()
}
