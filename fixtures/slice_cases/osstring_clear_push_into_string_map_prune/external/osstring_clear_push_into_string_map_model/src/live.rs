use std::ffi::OsString;
pub struct OsstringClearPushIntoStringMapPayload {
    value: String,
}

impl OsstringClearPushIntoStringMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("osstring-clear-push-into-string-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("osstring-clear-push-into-string-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-osstring-clear-push-into-string-map:{}", self.value)
    }
}

pub fn selected_osstring_clear_push_into_string_map(raw: &str) -> String {
    let mut value = OsString::from("dead");
    value.clear();
    value.push(raw);
    value
        .into_string()
        .ok()
        .map(|text| OsstringClearPushIntoStringMapPayload::new(&text).render_label())
        .unwrap_or_default()
}

pub fn dead_live_osstring_clear_push_into_string_map(raw: &str) -> String {
    OsstringClearPushIntoStringMapPayload::new(raw).unused_label()
}
