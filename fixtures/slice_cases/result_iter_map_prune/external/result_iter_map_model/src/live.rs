#[derive(Clone)]
pub struct ResultIterMapPayload {
    value: String,
}

impl ResultIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-iter-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-iter-map:{}", self.value)
    }
}

#[derive(Clone)]
pub struct ResultIterMapError {
    value: String,
}

impl ResultIterMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-iter-map-error:{}", self.value)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-result-iter-map-error:{}", self.value)
    }
}

fn result_iter_map_payload(raw: &str) -> Result<ResultIterMapPayload, ResultIterMapError> {
    if raw.is_empty() {
        Err(ResultIterMapError::new(raw))
    } else {
        Ok(ResultIterMapPayload::new(raw))
    }
}

pub fn selected_result_iter_map(raw: &str) -> String {
    result_iter_map_payload(raw)
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| ResultIterMapError::new(raw).render_error())
}

pub fn dead_live_result_iter_map(raw: &str) -> String {
    ResultIterMapPayload::new(raw).dead_method()
}
