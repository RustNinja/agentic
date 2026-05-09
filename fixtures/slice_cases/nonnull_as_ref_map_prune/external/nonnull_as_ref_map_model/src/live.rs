use std::ptr::NonNull;

#[derive(Clone)]
pub struct NonNullAsRefMapPayload {
    value: String,
}

impl NonNullAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("nonnull-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("nonnull-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nonnull-as-ref-map:{}", self.value)
    }
}

pub fn selected_nonnull_as_ref_map(raw: &str) -> String {
    let payload = NonNullAsRefMapPayload::new(raw);
    let pointer: NonNull<NonNullAsRefMapPayload> = NonNull::from(&payload);
    unsafe { pointer.as_ref().render_label() }
}

pub fn dead_live_nonnull_as_ref_map(raw: &str) -> String {
    NonNullAsRefMapPayload::new(raw).dead_method()
}
