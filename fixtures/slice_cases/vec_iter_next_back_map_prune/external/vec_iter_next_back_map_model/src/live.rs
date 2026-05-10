pub struct VecIterNextBackMapPayload {
    value: String,
}

impl VecIterNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-iter-next-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-iter-next-back-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-iter-next-back-map:{}", self.value)
    }
}

fn vec_iter_next_back_map_items(raw: &str) -> Vec<VecIterNextBackMapPayload> {
    vec![VecIterNextBackMapPayload::new("head"), VecIterNextBackMapPayload::new(raw)]
}

pub fn selected_vec_iter_next_back_map(raw: &str) -> String {
    vec_iter_next_back_map_items(raw)
        .iter()
        .next_back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vec-iter-next-back-map:missing".to_string())
}

pub fn dead_live_vec_iter_next_back_map(raw: &str) -> String {
    VecIterNextBackMapPayload::new(raw).unused_label()
}
