pub struct VecRemovePayload {
    value: String,
}

impl VecRemovePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-remove:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-remove:{}", self.value)
    }
}

fn vec_remove_items(raw: &str) -> Vec<VecRemovePayload> {
    vec![VecRemovePayload::new(raw), VecRemovePayload::new("tail")]
}

pub fn selected_vec_remove(raw: &str) -> String {
    let mut payloads = vec_remove_items(raw);
    payloads.remove(0).render_label()
}

pub fn dead_live_vec_remove(raw: &str) -> String {
    VecRemovePayload::new(raw).dead_method()
}
