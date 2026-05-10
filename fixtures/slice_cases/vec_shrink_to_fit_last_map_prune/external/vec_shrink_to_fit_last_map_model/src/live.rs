pub struct VecShrinkToFitLastMapPayload {
    value: String,
}

impl VecShrinkToFitLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-shrink-to-fit-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-shrink-to-fit-last-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-shrink-to-fit-last-map:{}", self.value)
    }
}

pub fn selected_vec_shrink_to_fit_last_map(raw: &str) -> String {
    let mut values = vec![VecShrinkToFitLastMapPayload::new(raw)];
    values.shrink_to_fit();
    values
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vec_shrink_to_fit_last_map(raw: &str) -> String {
    VecShrinkToFitLastMapPayload::new(raw).unused_label()
}
