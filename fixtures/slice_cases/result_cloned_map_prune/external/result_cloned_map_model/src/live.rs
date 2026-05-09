#[derive(Clone)]
pub struct ResultClonedMapPayload {
    value: String,
}

impl ResultClonedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-cloned-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-cloned-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-cloned-map:{}", self.value)
    }
}

pub struct ResultClonedMapError {
    value: String,
}

impl ResultClonedMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-cloned-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-cloned-map-error:{}", self.value)
    }
}

pub fn selected_result_cloned_map(raw: &str) -> String {
    let payload = ResultClonedMapPayload::new(raw);
    let result: Result<&ResultClonedMapPayload, ResultClonedMapError> = Ok(&payload);
    result
        .cloned()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_cloned_map(raw: &str) -> String {
    let mut payload = ResultClonedMapPayload::new(raw);
    payload.bump_and_render()
}
