use std::mem;

#[derive(Clone, Default)]
pub struct MemTakeMapPayload {
    value: String,
}

impl MemTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mem-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("mem-take-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mem-take-map:{}", self.value)
    }
}

pub fn selected_mem_take_map(raw: &str) -> String {
    let mut payload = MemTakeMapPayload::new(raw);
    mem::take(&mut payload).render_label()
}

pub fn dead_live_mem_take_map(raw: &str) -> String {
    MemTakeMapPayload::new(raw).dead_method()
}
