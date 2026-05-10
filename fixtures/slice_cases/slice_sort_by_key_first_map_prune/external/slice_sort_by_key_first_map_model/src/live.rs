pub struct SliceSortByKeyFirstMapPayload {
    value: String,
}

impl SliceSortByKeyFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-sort-by-key-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-sort-by-key-first-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-sort-by-key-first-map:{}", self.value)
    }

    pub fn rank(&self) -> usize {
        self.value.len()
    }
}

pub fn selected_slice_sort_by_key_first_map(raw: &str) -> String {
    let mut values = [
        SliceSortByKeyFirstMapPayload::new(raw),
        SliceSortByKeyFirstMapPayload::new("zz"),
    ];
    values.as_mut_slice().sort_by_key(|payload| payload.rank());
    values
        .first()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_slice_sort_by_key_first_map(raw: &str) -> String {
    SliceSortByKeyFirstMapPayload::new(raw).unused_label()
}
