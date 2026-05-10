#[derive(Clone)]
pub struct VecExtendFromWithinMapPayload {
    value: String,
}

impl VecExtendFromWithinMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-extend-from-within-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-extend-from-within-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-extend-from-within-map:{}", self.value)
    }
}

pub fn selected_vec_extend_from_within_map(raw: &str) -> String {
    let mut values = vec![VecExtendFromWithinMapPayload::new(raw)];
    values.extend_from_within(0..1);
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_extend_from_within_map(raw: &str) -> String {
    VecExtendFromWithinMapPayload::new(raw).unused_label()
}
