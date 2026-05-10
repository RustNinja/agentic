use std::iter;
#[derive(Clone)]
pub struct IterRepeatNMapPayload {
    value: String,
}

impl IterRepeatNMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iter-repeat-n-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iter-repeat-n-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iter-repeat-n-map:{}", self.value)
    }
}

pub fn selected_iter_repeat_n_map(raw: &str) -> String {
    iter::repeat_n(IterRepeatNMapPayload::new(raw), 1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_iter_repeat_n_map(raw: &str) -> String {
    IterRepeatNMapPayload::new(raw).unused_label()
}
