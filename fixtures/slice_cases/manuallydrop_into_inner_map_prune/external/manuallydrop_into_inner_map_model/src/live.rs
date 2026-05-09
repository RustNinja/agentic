use std::mem::ManuallyDrop;

#[derive(Clone)]
pub struct ManuallyDropIntoInnerMapPayload {
    value: String,
}

impl ManuallyDropIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("manuallydrop-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("manuallydrop-into-inner-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-manuallydrop-into-inner-map:{}", self.value)
    }
}

pub fn selected_manuallydrop_into_inner_map(raw: &str) -> String {
    let payload: ManuallyDrop<ManuallyDropIntoInnerMapPayload> =
        ManuallyDrop::new(ManuallyDropIntoInnerMapPayload::new(raw));
    ManuallyDrop::into_inner(payload).render_label()
}

pub fn dead_live_manuallydrop_into_inner_map(raw: &str) -> String {
    ManuallyDropIntoInnerMapPayload::new(raw).dead_method()
}
