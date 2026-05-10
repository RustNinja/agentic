use std::sync::Mutex;
#[derive(Debug)]
pub struct MutexTryLockUnwrapMapPayload {
    value: String,
}

impl MutexTryLockUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-try-lock-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mutex-try-lock-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mutex-try-lock-unwrap-map:{}", self.value)
    }
}

pub fn selected_mutex_try_lock_unwrap_map(raw: &str) -> String {
    let payload = Mutex::new(MutexTryLockUnwrapMapPayload::new(raw));
    let label = payload.try_lock().unwrap().render_label();
    label
}

pub fn dead_live_mutex_try_lock_unwrap_map(raw: &str) -> String {
    MutexTryLockUnwrapMapPayload::new(raw).unused_label()
}
