use std::borrow::Cow;

#[derive(Clone)]
pub struct CowToMutMapPayload {
    value: String,
}

impl CowToMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cow-to-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("cow-to-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cow-to-mut-map:{}", self.value)
    }
}

pub fn selected_cow_to_mut_map(raw: &str) -> String {
    let mut cow: Cow<'_, CowToMutMapPayload> = Cow::Owned(CowToMutMapPayload::new(raw));
    cow.to_mut().bump_and_render()
}

pub fn dead_live_cow_to_mut_map(raw: &str) -> String {
    CowToMutMapPayload::new(raw).dead_method()
}
