use std::sync::Mutex;
pub struct MutexGetMutMapPayload {
    value: String,
}

impl MutexGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mutex-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mutex-get-mut-map:{}", self.value)
    }
}

pub fn selected_mutex_get_mut_map(raw: &str) -> String {
    let mut payload = Mutex::new(MutexGetMutMapPayload::new(raw));
    payload
        .get_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_mutex_get_mut_map(raw: &str) -> String {
    MutexGetMutMapPayload::new(raw).unused_label()
}
