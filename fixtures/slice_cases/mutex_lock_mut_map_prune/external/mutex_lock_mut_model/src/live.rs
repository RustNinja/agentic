use std::sync::Mutex;

pub struct MutexLockMutMapPayload {
    value: String,
}

impl MutexLockMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-lock-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("mutex-lock-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-lock-mut-map:{}", self.value)
    }
}

pub fn selected_mutex_lock_mut_map(raw: &str) -> String {
    let payload = Mutex::new(MutexLockMutMapPayload::new(raw));
    let mut guard = payload.lock().expect("fixture mutex should not poison");
    guard.bump_and_render()
}

pub fn dead_live_mutex_lock_mut_map(raw: &str) -> String {
    MutexLockMutMapPayload::new(raw).dead_method()
}
