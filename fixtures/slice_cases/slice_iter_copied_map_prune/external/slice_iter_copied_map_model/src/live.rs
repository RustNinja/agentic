#[derive(Clone, Copy)]
pub struct SliceIterCopiedMapPayload {
    value: usize,
}

impl SliceIterCopiedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-iter-copied-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-iter-copied-map:{}", self.value)
    }
}

pub fn selected_slice_iter_copied_map(raw: &str) -> String {
    let items = [SliceIterCopiedMapPayload::new(raw)];
    let slice = &items[..];
    slice
        .iter()
        .copied()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("slice-iter-copied-map:missing"))
}

pub fn dead_live_slice_iter_copied_map(raw: &str) -> String {
    SliceIterCopiedMapPayload::new(raw).dead_method()
}
