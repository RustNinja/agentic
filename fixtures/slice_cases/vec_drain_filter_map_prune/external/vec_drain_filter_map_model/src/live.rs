pub struct VecDrainFilterMapPayload {
    value: String,
}

impl VecDrainFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("vec-drain-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-drain-filter-map:{}", self.value)
    }
}

pub fn selected_vec_drain_filter_map(raw: &str) -> String {
    let mut items = vec![
        VecDrainFilterMapPayload::new(""),
        VecDrainFilterMapPayload::new(raw),
    ];
    items
        .drain(..)
        .filter_map(VecDrainFilterMapPayload::maybe_label)
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_drain_filter_map(raw: &str) -> String {
    VecDrainFilterMapPayload::new(raw).unused_label()
}
