use std::path::PathBuf;
pub struct PathbufClearPushMapPayload {
    value: String,
}

impl PathbufClearPushMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-clear-push-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-clear-push-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-clear-push-map:{}", self.value)
    }
}

pub fn selected_pathbuf_clear_push_map(raw: &str) -> String {
    let mut path = PathBuf::from("dead");
    path.clear();
    path.push(raw);
    path.to_str()
        .map(PathbufClearPushMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_clear_push_map(raw: &str) -> String {
    PathbufClearPushMapPayload::new(raw).unused_label()
}
