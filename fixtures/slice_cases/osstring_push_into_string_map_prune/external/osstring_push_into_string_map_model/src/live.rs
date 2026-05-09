use std::ffi::OsString;
pub struct OsstringPushIntoStringMapPayload {
    value: String,
}

impl OsstringPushIntoStringMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("osstring-push-into-string-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("osstring-push-into-string-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-osstring-push-into-string-map:{}", self.value)
    }
}

pub fn selected_osstring_push_into_string_map(raw: &str) -> String {
    let mut value = OsString::from(raw);
    value.push("tail");
    value
        .into_string()
        .ok()
        .map(|text| OsstringPushIntoStringMapPayload::new(&text).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_osstring_push_into_string_map(raw: &str) -> String {
    OsstringPushIntoStringMapPayload::new(raw).unused_label()
}
