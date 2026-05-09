#[derive(Clone)]
pub struct OptionTransposeUnwrapMapPayload {
    value: String,
}

impl OptionTransposeUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-transpose-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-transpose-unwrap-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-transpose-unwrap-map:{}", self.value)
    }
}

pub struct OptionTransposeUnwrapMapError {
    value: String,
}

impl OptionTransposeUnwrapMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("option-transpose-unwrap-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-transpose-unwrap-map-error:{}", self.value)
    }
}

pub fn selected_option_transpose_unwrap_map(raw: &str) -> String {
    let optional: Option<Result<OptionTransposeUnwrapMapPayload, OptionTransposeUnwrapMapError>> =
        Some(Ok(OptionTransposeUnwrapMapPayload::new(raw)));
    optional
        .transpose()
        .unwrap_or_else(|err| {
            let _ = err.render_error();
            None
        })
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| format!("option-transpose-unwrap-map:missing"))
}

pub fn dead_live_option_transpose_unwrap_map(raw: &str) -> String {
    let mut payload = OptionTransposeUnwrapMapPayload::new(raw);
    payload.bump_and_render()
}
