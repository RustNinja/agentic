#[derive(Clone)]
pub struct VecIntoIterTakeMapPayload {
    value: String,
}

impl VecIntoIterTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-into-iter-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-into-iter-take-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-into-iter-take-map:{}", self.value)
    }
}

pub fn selected_vec_into_iter_take_map(raw: &str) -> String {
    let items = vec![
        VecIntoIterTakeMapPayload::new(raw),
        VecIntoIterTakeMapPayload::new("dead"),
    ];
    items
        .into_iter()
        .take(1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("vec-into-iter-take-map:missing"))
}

pub fn dead_live_vec_into_iter_take_map(raw: &str) -> String {
    let mut payload = VecIntoIterTakeMapPayload::new(raw);
    payload.bump_and_render()
}
