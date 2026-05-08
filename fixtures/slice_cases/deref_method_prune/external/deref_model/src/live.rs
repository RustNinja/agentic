use std::ops::Deref;

pub struct DerefInner {
    label: String,
}

impl DerefInner {
    pub fn render(&self) -> String {
        format!("deref:{}", self.label)
    }

    pub fn dead_inner_method(&self) -> String {
        format!("dead-inner:{}", self.label)
    }
}

pub struct DerefWrapper {
    inner: DerefInner,
}

impl DerefWrapper {
    pub fn new(raw: &str) -> Self {
        Self {
            inner: DerefInner {
                label: raw.trim().to_string(),
            },
        }
    }

    pub fn dead_wrapper_method(&self) -> String {
        self.inner.dead_inner_method()
    }
}

impl Deref for DerefWrapper {
    type Target = DerefInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub fn selected_deref(raw: &str) -> String {
    DerefWrapper::new(raw).render()
}

pub fn dead_live_deref(raw: &str) -> String {
    DerefWrapper::new(raw).dead_wrapper_method()
}
