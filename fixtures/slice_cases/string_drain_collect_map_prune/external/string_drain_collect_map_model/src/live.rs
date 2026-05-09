pub struct StringDrainCollectMapPayload {
    value: String,
}

impl StringDrainCollectMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-drain-collect-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-drain-collect-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-drain-collect-map:{}", self.value)
    }
}

pub fn selected_string_drain_collect_map(raw: &str) -> String {
    let mut value = raw.to_string();
    value.push_str(":tail");
    let drained = value.drain(..).collect::<String>();
    StringDrainCollectMapPayload::new(&drained).render_label()
}

pub fn dead_live_string_drain_collect_map(raw: &str) -> String {
    StringDrainCollectMapPayload::new(raw).unused_label()
}
