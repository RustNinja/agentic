use std::path::PathBuf;
pub struct PathbufReservePushMapPayload {
    value: String,
}

impl PathbufReservePushMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-reserve-push-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-reserve-push-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-reserve-push-map:{}", self.value)
    }
}

pub fn selected_pathbuf_reserve_push_map(raw: &str) -> String {
    let mut path = PathBuf::with_capacity(raw.len() + 4);
    path.reserve(4);
    path.push(raw);
    path.to_str()
        .map(PathbufReservePushMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_reserve_push_map(raw: &str) -> String {
    PathbufReservePushMapPayload::new(raw).unused_label()
}
