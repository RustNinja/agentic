pub struct AsMutTraitMapPayload {
    value: String,
}

impl AsMutTraitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("as-mut-trait-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("as-mut-trait-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-as-mut-trait-map:{}", self.value)
    }
}

pub struct AsMutTraitMapWrapper {
    inner: AsMutTraitMapPayload,
}

impl AsMutTraitMapWrapper {
    pub fn new(raw: &str) -> Self {
        Self {
            inner: AsMutTraitMapPayload::new(raw),
        }
    }

    pub fn dead_wrapper_method(&self) -> String {
        self.inner.dead_method()
    }
}

impl AsMut<AsMutTraitMapPayload> for AsMutTraitMapWrapper {
    fn as_mut(&mut self) -> &mut AsMutTraitMapPayload {
        &mut self.inner
    }
}

pub fn selected_as_mut_trait_map(raw: &str) -> String {
    let mut wrapper = AsMutTraitMapWrapper::new(raw);
    wrapper.as_mut().bump_and_render()
}

pub fn dead_live_as_mut_trait_map(raw: &str) -> String {
    AsMutTraitMapPayload::new(raw).dead_method()
}
