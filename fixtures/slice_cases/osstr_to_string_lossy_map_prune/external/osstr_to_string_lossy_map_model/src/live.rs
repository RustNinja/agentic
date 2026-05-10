use std::ffi::OsStr;
pub struct OsstrToStringLossyMapPayload {
    value: String,
}

impl OsstrToStringLossyMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("osstr-to-string-lossy-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("osstr-to-string-lossy-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-osstr-to-string-lossy-map:{}", self.value)
    }
}

pub fn selected_osstr_to_string_lossy_map(raw: &str) -> String {
    let value = OsStr::new(raw).to_string_lossy();
    OsstrToStringLossyMapPayload::new(&value).render_label()
}

pub fn dead_live_osstr_to_string_lossy_map(raw: &str) -> String {
    OsstrToStringLossyMapPayload::new(raw).unused_label()
}
