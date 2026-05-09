use std::path::PathBuf;
pub struct PathbufPopMapPayload {
    value: String,
}

impl PathbufPopMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-pop-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-pop-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-pop-map:{}", self.value)
    }
}

pub fn selected_pathbuf_pop_map(raw: &str) -> String {
    let mut path = PathBuf::from(raw);
    path.push("tail");
    if path.pop() {
        path.to_str()
            .map(PathbufPopMapPayload::new)
            .map(|payload| payload.render_label())
            .unwrap_or_else(|| "missing".to_string())
    } else {
        "missing".to_string()
    }
}

pub fn dead_live_pathbuf_pop_map(raw: &str) -> String {
    PathbufPopMapPayload::new(raw).unused_label()
}
