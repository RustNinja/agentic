use std::sync::{Arc, Mutex};

pub struct ArcMutexLockMapPayload {
    value: String,
}

impl ArcMutexLockMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-mutex-lock-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("arc-mutex-lock-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-mutex-lock-map:{}", self.value)
    }
}

pub fn selected_arc_mutex_lock_map(raw: &str) -> String {
    let payload = Arc::new(Mutex::new(ArcMutexLockMapPayload::new(raw)));
    let guard = payload.lock().expect("fixture arc mutex should not poison");
    guard.render_label()
}

pub fn dead_live_arc_mutex_lock_map(raw: &str) -> String {
    ArcMutexLockMapPayload::new(raw).dead_method()
}
