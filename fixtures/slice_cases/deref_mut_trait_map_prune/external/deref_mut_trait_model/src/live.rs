use std::ops::{Deref, DerefMut};

pub struct DerefMutTraitMapPayload {
    value: String,
}

impl DerefMutTraitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("deref-mut-trait-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("deref-mut-trait-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-deref-mut-trait-map:{}", self.value)
    }
}

pub struct DerefMutTraitMapWrapper {
    inner: DerefMutTraitMapPayload,
}

impl DerefMutTraitMapWrapper {
    pub fn new(raw: &str) -> Self {
        Self {
            inner: DerefMutTraitMapPayload::new(raw),
        }
    }

    pub fn dead_wrapper_method(&self) -> String {
        self.inner.dead_method()
    }
}

impl Deref for DerefMutTraitMapWrapper {
    type Target = DerefMutTraitMapPayload;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DerefMutTraitMapWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn selected_deref_mut_trait_map(raw: &str) -> String {
    let mut wrapper = DerefMutTraitMapWrapper::new(raw);
    wrapper.bump_and_render()
}

pub fn dead_live_deref_mut_trait_map(raw: &str) -> String {
    DerefMutTraitMapPayload::new(raw).dead_method()
}
