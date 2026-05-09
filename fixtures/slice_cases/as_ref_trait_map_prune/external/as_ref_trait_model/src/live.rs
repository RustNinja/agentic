pub struct AsRefTraitMapPayload {
    value: String,
}

impl AsRefTraitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("as-ref-trait-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("as-ref-trait-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-as-ref-trait-map:{}", self.value)
    }
}

pub struct AsRefTraitMapWrapper {
    inner: AsRefTraitMapPayload,
}

impl AsRefTraitMapWrapper {
    pub fn new(raw: &str) -> Self {
        Self {
            inner: AsRefTraitMapPayload::new(raw),
        }
    }

    pub fn dead_wrapper_method(&self) -> String {
        self.inner.dead_method()
    }
}

impl AsRef<AsRefTraitMapPayload> for AsRefTraitMapWrapper {
    fn as_ref(&self) -> &AsRefTraitMapPayload {
        &self.inner
    }
}

pub fn selected_as_ref_trait_map(raw: &str) -> String {
    let wrapper = AsRefTraitMapWrapper::new(raw);
    wrapper.as_ref().render_label()
}

pub fn dead_live_as_ref_trait_map(raw: &str) -> String {
    AsRefTraitMapPayload::new(raw).dead_method()
}
