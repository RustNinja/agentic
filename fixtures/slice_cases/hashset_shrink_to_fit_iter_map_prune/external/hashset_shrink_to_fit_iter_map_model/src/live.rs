use std::collections::HashSet;
pub struct HashsetShrinkToFitIterMapPayload {
    value: String,
}

impl HashsetShrinkToFitIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-shrink-to-fit-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashset-shrink-to-fit-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashset-shrink-to-fit-iter-map:{}", self.value)
    }
}

pub fn selected_hashset_shrink_to_fit_iter_map(raw: &str) -> String {
    let mut values = HashSet::new();
    values.insert(raw.to_string());
    values.shrink_to_fit();
    values
        .iter()
        .map(|value| HashsetShrinkToFitIterMapPayload::new(value).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_hashset_shrink_to_fit_iter_map(raw: &str) -> String {
    HashsetShrinkToFitIterMapPayload::new(raw).unused_label()
}
