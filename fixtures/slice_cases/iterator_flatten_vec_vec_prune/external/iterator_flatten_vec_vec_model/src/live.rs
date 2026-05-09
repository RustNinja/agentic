pub struct IteratorFlattenVecVecPayload {
    value: String,
}

impl IteratorFlattenVecVecPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-vec-vec:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-vec-vec:{}", self.value)
    }
}

pub fn selected_iterator_flatten_vec_vec(raw: &str) -> String {
    let groups: Vec<Vec<IteratorFlattenVecVecPayload>> =
        vec![vec![IteratorFlattenVecVecPayload::new(raw)], Vec::new()];
    groups
        .into_iter()
        .flatten()
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_vec_vec(raw: &str) -> String {
    IteratorFlattenVecVecPayload::new(raw).dead_method()
}
