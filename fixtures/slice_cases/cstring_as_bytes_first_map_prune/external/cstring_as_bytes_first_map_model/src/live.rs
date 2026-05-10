use std::ffi::CString;
pub struct CstringAsBytesFirstMapPayload {
    value: String,
}

impl CstringAsBytesFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cstring-as-bytes-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("cstring-as-bytes-first-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-cstring-as-bytes-first-map:{}", self.value)
    }
}

pub fn selected_cstring_as_bytes_first_map(raw: &str) -> String {
    let value = CString::new(raw).unwrap_or_else(|_| CString::new("fallback").unwrap());
    value
        .as_bytes()
        .first()
        .map(|byte| CstringAsBytesFirstMapPayload::new(&byte.to_string()).render_label())
        .unwrap_or_default()
}

pub fn dead_live_cstring_as_bytes_first_map(raw: &str) -> String {
    CstringAsBytesFirstMapPayload::new(raw).unused_label()
}
