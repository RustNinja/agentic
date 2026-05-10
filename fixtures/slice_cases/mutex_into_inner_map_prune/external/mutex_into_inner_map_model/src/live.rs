use std::sync::Mutex;
pub struct MutexIntoInnerMapPayload {
    value: String,
}

impl MutexIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mutex-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mutex-into-inner-map:{}", self.value)
    }
}

pub fn selected_mutex_into_inner_map(raw: &str) -> String {
    let payload = Mutex::new(MutexIntoInnerMapPayload::new(raw));
    Mutex::into_inner(payload)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mutex_into_inner_map(raw: &str) -> String {
    MutexIntoInnerMapPayload::new(raw).unused_label()
}
