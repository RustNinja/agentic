use std::path::PathBuf;
pub struct PathbufPushToStrMapPayload {
    value: String,
}

impl PathbufPushToStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-push-to-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-push-to-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-push-to-str-map:{}", self.value)
    }
}

pub fn selected_pathbuf_push_to_str_map(raw: &str) -> String {
    let mut path = PathBuf::from(raw);
    path.push("tail");
    path.to_str()
        .map(PathbufPushToStrMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_push_to_str_map(raw: &str) -> String {
    PathbufPushToStrMapPayload::new(raw).unused_label()
}
