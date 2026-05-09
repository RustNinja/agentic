#[derive(Clone)]
pub struct OptionAsMutSliceIterMutPayload {
    value: String,
}

impl OptionAsMutSliceIterMutPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-as-mut-slice-iter-mut:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-as-mut-slice-iter-mut:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-mut-slice-iter-mut:{}", self.value)
    }
}

fn option_as_mut_slice_iter_mut_payload(raw: &str) -> Option<OptionAsMutSliceIterMutPayload> {
    Some(OptionAsMutSliceIterMutPayload::new(raw))
}

pub fn selected_option_as_mut_slice_iter_mut(raw: &str) -> String {
    let mut payload = option_as_mut_slice_iter_mut_payload(raw);
    payload
        .as_mut_slice()
        .iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "option-as-mut-slice-iter-mut:missing".to_string())
}

pub fn dead_live_option_as_mut_slice_iter_mut(raw: &str) -> String {
    OptionAsMutSliceIterMutPayload::new(raw).dead_method()
}
