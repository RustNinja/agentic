use std::sync::Mutex;
#[derive(Debug)]
pub struct MutexIntoInnerUnwrapMapPayload {
    value: String,
}

impl MutexIntoInnerUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mutex-into-inner-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mutex-into-inner-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mutex-into-inner-unwrap-map:{}", self.value)
    }
}

pub fn selected_mutex_into_inner_unwrap_map(raw: &str) -> String {
    let payload = Mutex::new(MutexIntoInnerUnwrapMapPayload::new(raw));
    Mutex::into_inner(payload).unwrap().render_label()
}

pub fn dead_live_mutex_into_inner_unwrap_map(raw: &str) -> String {
    MutexIntoInnerUnwrapMapPayload::new(raw).unused_label()
}
