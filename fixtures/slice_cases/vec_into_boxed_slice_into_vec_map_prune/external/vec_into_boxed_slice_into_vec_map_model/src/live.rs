pub struct VecIntoBoxedSliceIntoVecMapPayload {
    value: String,
}

impl VecIntoBoxedSliceIntoVecMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-into-boxed-slice-into-vec-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-into-boxed-slice-into-vec-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-into-boxed-slice-into-vec-map:{}", self.value)
    }
}

pub fn selected_vec_into_boxed_slice_into_vec_map(raw: &str) -> String {
    let boxed = vec![VecIntoBoxedSliceIntoVecMapPayload::new(raw)].into_boxed_slice();
    boxed
        .into_vec()
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_vec_into_boxed_slice_into_vec_map(raw: &str) -> String {
    VecIntoBoxedSliceIntoVecMapPayload::new(raw).unused_label()
}
