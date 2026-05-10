pub struct BoxedSliceIterMapPayload {
    value: String,
}

impl BoxedSliceIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("boxed-slice-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("boxed-slice-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-boxed-slice-iter-map:{}", self.value)
    }
}

pub fn selected_boxed_slice_iter_map(raw: &str) -> String {
    let boxed: Box<[BoxedSliceIterMapPayload]> = Box::new([BoxedSliceIterMapPayload::new(raw)]);
    boxed
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_boxed_slice_iter_map(raw: &str) -> String {
    BoxedSliceIterMapPayload::new(raw).unused_label()
}
