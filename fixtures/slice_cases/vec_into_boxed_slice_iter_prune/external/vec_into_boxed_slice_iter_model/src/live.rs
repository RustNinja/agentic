#[derive(Clone)]
pub struct VecIntoBoxedSliceIterPayload {
    value: String,
}

impl VecIntoBoxedSliceIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-into-boxed-slice-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-into-boxed-slice-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-into-boxed-slice-iter:{}", self.value)
    }
}

pub fn selected_vec_into_boxed_slice_iter(raw: &str) -> String {
    let payloads = vec![
        VecIntoBoxedSliceIterPayload::new(raw),
        VecIntoBoxedSliceIterPayload::new("tail"),
    ];
    payloads
        .into_boxed_slice()
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-into-boxed-slice-iter:missing".to_string())
}

pub fn dead_live_vec_into_boxed_slice_iter(raw: &str) -> String {
    VecIntoBoxedSliceIterPayload::new(raw).dead_method()
}
