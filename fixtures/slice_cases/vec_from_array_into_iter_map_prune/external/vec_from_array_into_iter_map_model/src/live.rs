pub struct VecFromArrayIntoIterMapPayload {
    value: String,
}

impl VecFromArrayIntoIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-from-array-into-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-from-array-into-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-from-array-into-iter-map:{}", self.value)
    }
}

pub fn selected_vec_from_array_into_iter_map(raw: &str) -> String {
    Vec::from([VecFromArrayIntoIterMapPayload::new(raw)])
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_vec_from_array_into_iter_map(raw: &str) -> String {
    VecFromArrayIntoIterMapPayload::new(raw).unused_label()
}
