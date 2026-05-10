pub struct VecWindowsMapPayload {
    value: String,
}

impl VecWindowsMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-windows-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-windows-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-windows-map:{}", self.value)
    }
}

pub fn selected_vec_windows_map(raw: &str) -> String {
    let values = vec![
        VecWindowsMapPayload::new(raw),
        VecWindowsMapPayload::new("tail"),
    ];
    values
        .windows(1)
        .filter_map(|chunk| chunk.first())
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_windows_map(raw: &str) -> String {
    VecWindowsMapPayload::new(raw).unused_label()
}
