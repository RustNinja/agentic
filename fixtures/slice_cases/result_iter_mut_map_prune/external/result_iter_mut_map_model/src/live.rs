#[derive(Clone)]
pub struct ResultIterMutMapPayload {
    value: String,
}

impl ResultIterMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-iter-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-iter-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-iter-mut-map:{}", self.value)
    }
}

#[derive(Clone)]
pub struct ResultIterMutMapError {
    value: String,
}

impl ResultIterMutMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-iter-mut-map-error:{}", self.value)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-result-iter-mut-map-error:{}", self.value)
    }
}

fn result_iter_mut_map_payload(
    raw: &str,
) -> Result<ResultIterMutMapPayload, ResultIterMutMapError> {
    if raw.is_empty() {
        Err(ResultIterMutMapError::new(raw))
    } else {
        Ok(ResultIterMutMapPayload::new(raw))
    }
}

pub fn selected_result_iter_mut_map(raw: &str) -> String {
    let mut payload = result_iter_mut_map_payload(raw);
    payload
        .iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| ResultIterMutMapError::new(raw).render_error())
}

pub fn dead_live_result_iter_mut_map(raw: &str) -> String {
    ResultIterMutMapPayload::new(raw).dead_method()
}
