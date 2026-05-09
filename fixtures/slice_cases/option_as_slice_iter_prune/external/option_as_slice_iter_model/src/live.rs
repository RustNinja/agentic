#[derive(Clone)]
pub struct OptionAsSliceIterPayload {
    value: String,
}

impl OptionAsSliceIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-as-slice-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-as-slice-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-slice-iter:{}", self.value)
    }
}

fn option_as_slice_iter_payload(raw: &str) -> Option<OptionAsSliceIterPayload> {
    Some(OptionAsSliceIterPayload::new(raw))
}

pub fn selected_option_as_slice_iter(raw: &str) -> String {
    option_as_slice_iter_payload(raw)
        .as_slice()
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "option-as-slice-iter:missing".to_string())
}

pub fn dead_live_option_as_slice_iter(raw: &str) -> String {
    OptionAsSliceIterPayload::new(raw).dead_method()
}
