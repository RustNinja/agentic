pub struct FuseLastPayload {
    value: String,
}

impl FuseLastPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("fuse-last:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-fuse-last:{}", self.value)
    }
}

fn fuse_last_items(raw: &str) -> Vec<FuseLastPayload> {
    vec![FuseLastPayload::new(raw)]
}

pub fn selected_fuse_last(raw: &str) -> String {
    fuse_last_items(raw)
        .into_iter()
        .fuse()
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "fuse-last:missing".to_string())
}

pub fn dead_live_fuse_last(raw: &str) -> String {
    FuseLastPayload::new(raw).dead_method()
}
