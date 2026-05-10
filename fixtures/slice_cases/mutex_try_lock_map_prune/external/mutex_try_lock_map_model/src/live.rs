use std::sync::Mutex;
pub struct MutexTryLockMapPayload {
    value: String,
}

impl MutexTryLockMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-try-lock-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mutex-try-lock-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mutex-try-lock-map:{}", self.value)
    }
}

pub fn selected_mutex_try_lock_map(raw: &str) -> String {
    let payload = Mutex::new(MutexTryLockMapPayload::new(raw));
    payload
        .try_lock()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mutex_try_lock_map(raw: &str) -> String {
    MutexTryLockMapPayload::new(raw).unused_label()
}
