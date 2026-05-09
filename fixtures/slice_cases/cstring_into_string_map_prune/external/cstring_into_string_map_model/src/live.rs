use std::ffi::CString;
pub struct CstringIntoStringMapPayload {
    value: String,
}

impl CstringIntoStringMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cstring-into-string-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("cstring-into-string-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-cstring-into-string-map:{}", self.value)
    }
}

pub fn selected_cstring_into_string_map(raw: &str) -> String {
    CString::new(raw)
        .ok()
        .and_then(|value| value.into_string().ok())
        .map(|value| CstringIntoStringMapPayload::new(&value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_cstring_into_string_map(raw: &str) -> String {
    CstringIntoStringMapPayload::new(raw).unused_label()
}
