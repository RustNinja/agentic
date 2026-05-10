use std::ptr::NonNull;
pub struct NonnullAsMutMapPayload {
    value: String,
}

impl NonnullAsMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("nonnull-as-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("nonnull-as-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-nonnull-as-mut-map:{}", self.value)
    }
}

pub fn selected_nonnull_as_mut_map(raw: &str) -> String {
    let leaked = Box::leak(Box::new(NonnullAsMutMapPayload::new(raw)));
    let mut ptr = NonNull::from(leaked);
    unsafe { ptr.as_mut().bump_and_render() }
}

pub fn dead_live_nonnull_as_mut_map(raw: &str) -> String {
    NonnullAsMutMapPayload::new(raw).unused_label()
}
