use std::path::PathBuf;
pub struct PathbufPushPopPushToStrMapPayload {
    value: String,
}

impl PathbufPushPopPushToStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-push-pop-push-to-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-push-pop-push-to-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-push-pop-push-to-str-map:{}", self.value)
    }
}

pub fn selected_pathbuf_push_pop_push_to_str_map(raw: &str) -> String {
    let mut path = PathBuf::from(raw);
    path.push("child");
    path.pop();
    path.push("live");
    path.to_str()
        .map(|value| PathbufPushPopPushToStrMapPayload::new(value).render_label())
        .unwrap_or_default()
}

pub fn dead_live_pathbuf_push_pop_push_to_str_map(raw: &str) -> String {
    PathbufPushPopPushToStrMapPayload::new(raw).unused_label()
}
