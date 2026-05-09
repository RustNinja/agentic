use std::ffi::OsStr;

pub struct OsstrToStrMapPayload {
    value: String,
}

impl OsstrToStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("osstr-to-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-osstr-to-str-map:{}", self.value)
    }
}

pub fn selected_osstr_to_str_map(raw: &str) -> String {
    OsStr::new(raw)
        .to_str()
        .map(OsstrToStrMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_osstr_to_str_map(raw: &str) -> String {
    OsstrToStrMapPayload::new(raw).unused_label()
}
