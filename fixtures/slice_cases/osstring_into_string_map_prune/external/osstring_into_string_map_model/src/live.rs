use std::ffi::OsString;
pub struct OsstringIntoStringMapPayload {
    value: String,
}

impl OsstringIntoStringMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("osstring-into-string-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("osstring-into-string-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-osstring-into-string-map:{}", self.value)
    }
}

pub fn selected_osstring_into_string_map(raw: &str) -> String {
    OsString::from(raw)
        .into_string()
        .ok()
        .map(|value| OsstringIntoStringMapPayload::new(&value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_osstring_into_string_map(raw: &str) -> String {
    OsstringIntoStringMapPayload::new(raw).unused_label()
}
