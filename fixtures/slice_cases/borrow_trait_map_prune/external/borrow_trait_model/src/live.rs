use std::borrow::Borrow;

pub struct BorrowTraitMapPayload {
    value: String,
}

impl BorrowTraitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("borrow-trait-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("borrow-trait-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-borrow-trait-map:{}", self.value)
    }
}

pub struct BorrowTraitMapWrapper {
    inner: BorrowTraitMapPayload,
}

impl BorrowTraitMapWrapper {
    pub fn new(raw: &str) -> Self {
        Self {
            inner: BorrowTraitMapPayload::new(raw),
        }
    }

    pub fn dead_wrapper_method(&self) -> String {
        self.inner.dead_method()
    }
}

impl Borrow<BorrowTraitMapPayload> for BorrowTraitMapWrapper {
    fn borrow(&self) -> &BorrowTraitMapPayload {
        &self.inner
    }
}

pub fn selected_borrow_trait_map(raw: &str) -> String {
    let wrapper = BorrowTraitMapWrapper::new(raw);
    let payload: &BorrowTraitMapPayload = wrapper.borrow();
    payload.render_label()
}

pub fn dead_live_borrow_trait_map(raw: &str) -> String {
    BorrowTraitMapPayload::new(raw).dead_method()
}
