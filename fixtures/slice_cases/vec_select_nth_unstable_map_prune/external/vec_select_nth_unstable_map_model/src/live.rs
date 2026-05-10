pub struct VecSelectNthUnstableMapPayload {
    value: String,
}

impl VecSelectNthUnstableMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-select-nth-unstable-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-select-nth-unstable-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-select-nth-unstable-map:{}", self.value)
    }
}

pub fn selected_vec_select_nth_unstable_map(raw: &str) -> String {
    let mut values = vec![
        VecSelectNthUnstableMapPayload::new("left"),
        VecSelectNthUnstableMapPayload::new(raw),
        VecSelectNthUnstableMapPayload::new("right"),
    ];
    let (_, payload, _) =
        values.select_nth_unstable_by(1, |left, right| left.value.cmp(&right.value));
    payload.render_label()
}

pub fn dead_live_vec_select_nth_unstable_map(raw: &str) -> String {
    VecSelectNthUnstableMapPayload::new(raw).unused_label()
}
