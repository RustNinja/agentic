#[derive(Clone)]
pub struct ResultAsMutMapPayload {
    value: String,
}

impl ResultAsMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-as-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-as-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-as-mut-map:{}", self.value)
    }
}

pub struct ResultAsMutMapError {
    value: String,
}

impl ResultAsMutMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-as-mut-map-error:{}", self.value)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-result-as-mut-map-error:{}", self.value)
    }
}
fn result_as_mut_map_payload(raw: &str) -> Result<ResultAsMutMapPayload, ResultAsMutMapError> {
    Ok(ResultAsMutMapPayload::new(raw))
}

pub fn selected_result_as_mut_map(raw: &str) -> String {
    let mut payload = result_as_mut_map_payload(raw);
    payload
        .as_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_as_mut_map(raw: &str) -> String {
    ResultAsMutMapPayload::new(raw).dead_method()
}
