use std::mem;

#[derive(Clone)]
pub struct MemReplaceMapPayload {
    value: String,
}

impl MemReplaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mem-replace-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("mem-replace-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mem-replace-map:{}", self.value)
    }
}

pub fn selected_mem_replace_map(raw: &str) -> String {
    let mut payload = MemReplaceMapPayload::new(raw);
    mem::replace(&mut payload, MemReplaceMapPayload::new("replacement")).render_label()
}

pub fn dead_live_mem_replace_map(raw: &str) -> String {
    MemReplaceMapPayload::new(raw).dead_method()
}
