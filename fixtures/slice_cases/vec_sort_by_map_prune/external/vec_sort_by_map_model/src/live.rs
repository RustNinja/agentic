pub struct VecSortByMapPayload {
    value: String,
}

impl VecSortByMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-sort-by-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-sort-by-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-sort-by-map:{}", self.value)
    }
}

pub fn selected_vec_sort_by_map(raw: &str) -> String {
    let mut values = vec![
        VecSortByMapPayload::new("tail"),
        VecSortByMapPayload::new(raw),
    ];
    values.sort_by(|left, right| left.value.cmp(&right.value));
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_sort_by_map(raw: &str) -> String {
    VecSortByMapPayload::new(raw).unused_label()
}
