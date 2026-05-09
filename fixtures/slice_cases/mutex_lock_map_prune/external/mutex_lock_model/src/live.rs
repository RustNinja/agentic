use std::sync::Mutex;

pub struct MutexLockMapPayload {
    value: String,
}

impl MutexLockMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-lock-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("mutex-lock-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-lock-map:{}", self.value)
    }
}

pub fn selected_mutex_lock_map(raw: &str) -> String {
    let payload = Mutex::new(MutexLockMapPayload::new(raw));
    let guard = payload.lock().expect("fixture mutex should not poison");
    guard.render_label()
}

pub fn dead_live_mutex_lock_map(raw: &str) -> String {
    MutexLockMapPayload::new(raw).dead_method()
}
