#[derive(Clone)]
pub struct IteratorByRefTakeMapPayload {
    value: String,
}

impl IteratorByRefTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-by-ref-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iterator-by-ref-take-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-by-ref-take-map:{}", self.value)
    }
}

pub fn selected_iterator_by_ref_take_map(raw: &str) -> String {
    let items = vec![
        IteratorByRefTakeMapPayload::new(raw),
        IteratorByRefTakeMapPayload::new("dead"),
    ];
    let mut iter = items.iter();
    iter.by_ref()
        .take(1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-by-ref-take-map:missing"))
}

pub fn dead_live_iterator_by_ref_take_map(raw: &str) -> String {
    let mut payload = IteratorByRefTakeMapPayload::new(raw);
    payload.bump_and_render()
}
