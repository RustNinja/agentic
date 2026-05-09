pub struct ResultAsDerefMutMapPayload {
    value: String,
}

impl ResultAsDerefMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-as-deref-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-as-deref-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-as-deref-mut-map:{}", self.value)
    }
}

pub struct ResultAsDerefMutMapError {
    code: String,
}

impl ResultAsDerefMutMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-as-deref-mut-map-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-result-as-deref-mut-map-error:{}", self.code)
    }
}

fn result_as_deref_mut_map_payload(raw: &str) -> Result<Box<ResultAsDerefMutMapPayload>, ResultAsDerefMutMapError> {
    if raw.trim().is_empty() {
        Err(ResultAsDerefMutMapError::new(raw))
    } else {
        Ok(Box::new(ResultAsDerefMutMapPayload::new(raw)))
    }
}

pub fn selected_result_as_deref_mut_map(raw: &str) -> String {
    let mut payload = result_as_deref_mut_map_payload(raw);
    payload
        .as_deref_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_as_deref_mut_map(raw: &str) -> String {
    ResultAsDerefMutMapPayload::new(raw).dead_method()
}
