#[derive(Clone)]
pub struct VecExtendFromSliceIterPayload {
    value: String,
}

impl VecExtendFromSliceIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-extend-from-slice-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-extend-from-slice-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-extend-from-slice-iter:{}", self.value)
    }
}

pub fn selected_vec_extend_from_slice_iter(raw: &str) -> String {
    let extras = [
        VecExtendFromSliceIterPayload::new(raw),
        VecExtendFromSliceIterPayload::new("tail"),
    ];
    let mut payloads = Vec::new();
    payloads.extend_from_slice(&extras);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-extend-from-slice-iter:missing".to_string())
}

pub fn dead_live_vec_extend_from_slice_iter(raw: &str) -> String {
    VecExtendFromSliceIterPayload::new(raw).dead_method()
}
