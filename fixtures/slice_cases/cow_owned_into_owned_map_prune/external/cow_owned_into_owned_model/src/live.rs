use std::borrow::Cow;

#[derive(Clone)]
pub struct CowOwnedIntoOwnedMapPayload {
    value: String,
}

impl CowOwnedIntoOwnedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cow-owned-into-owned-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("cow-owned-into-owned-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cow-owned-into-owned-map:{}", self.value)
    }
}

pub fn selected_cow_owned_into_owned_map(raw: &str) -> String {
    let cow: Cow<'_, CowOwnedIntoOwnedMapPayload> = Cow::Owned(CowOwnedIntoOwnedMapPayload::new(raw));
    cow.into_owned().render_label()
}

pub fn dead_live_cow_owned_into_owned_map(raw: &str) -> String {
    CowOwnedIntoOwnedMapPayload::new(raw).dead_method()
}
