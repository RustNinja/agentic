pub struct VecClearExtendMapPayload {
    value: String,
}

impl VecClearExtendMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-clear-extend-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-clear-extend-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-clear-extend-map:{}", self.value)
    }
}

pub fn selected_vec_clear_extend_map(raw: &str) -> String {
    let mut values = vec![VecClearExtendMapPayload::new("dead")];
    values.clear();
    values.extend([VecClearExtendMapPayload::new(raw)]);
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_clear_extend_map(raw: &str) -> String {
    VecClearExtendMapPayload::new(raw).unused_label()
}
