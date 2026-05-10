use std::ffi::CString;
pub struct CstrToStrMapPayload {
    value: String,
}

impl CstrToStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cstr-to-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("cstr-to-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-cstr-to-str-map:{}", self.value)
    }
}

pub fn selected_cstr_to_str_map(raw: &str) -> String {
    let value = CString::new(raw).unwrap_or_else(|_| CString::new("fallback").unwrap());
    value
        .as_c_str()
        .to_str()
        .ok()
        .map(|text| CstrToStrMapPayload::new(text).render_label())
        .unwrap_or_default()
}

pub fn dead_live_cstr_to_str_map(raw: &str) -> String {
    CstrToStrMapPayload::new(raw).unused_label()
}
