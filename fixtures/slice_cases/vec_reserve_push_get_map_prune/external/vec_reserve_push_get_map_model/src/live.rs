pub struct VecReservePushGetMapPayload {
    value: String,
}

impl VecReservePushGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-reserve-push-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-reserve-push-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-reserve-push-get-map:{}", self.value)
    }
}

pub fn selected_vec_reserve_push_get_map(raw: &str) -> String {
    let mut values = Vec::with_capacity(1);
    values.reserve(1);
    values.push(VecReservePushGetMapPayload::new(raw));
    values
        .get(0)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vec_reserve_push_get_map(raw: &str) -> String {
    VecReservePushGetMapPayload::new(raw).unused_label()
}
