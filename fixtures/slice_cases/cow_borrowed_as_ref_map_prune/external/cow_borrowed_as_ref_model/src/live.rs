use std::borrow::Cow;

#[derive(Clone)]
pub struct CowBorrowedAsRefMapPayload {
    value: String,
}

impl CowBorrowedAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cow-borrowed-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("cow-borrowed-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cow-borrowed-as-ref-map:{}", self.value)
    }
}

pub fn selected_cow_borrowed_as_ref_map(raw: &str) -> String {
    let payload = CowBorrowedAsRefMapPayload::new(raw);
    let cow: Cow<'_, CowBorrowedAsRefMapPayload> = Cow::Borrowed(&payload);
    cow.as_ref().render_label()
}

pub fn dead_live_cow_borrowed_as_ref_map(raw: &str) -> String {
    CowBorrowedAsRefMapPayload::new(raw).dead_method()
}
