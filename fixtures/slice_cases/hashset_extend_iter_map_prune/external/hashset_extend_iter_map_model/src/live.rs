use std::collections::HashSet;
pub struct HashsetExtendIterMapPayload {
    value: String,
}

impl HashsetExtendIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-extend-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashset-extend-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashset-extend-iter-map:{}", self.value)
    }
}

pub fn selected_hashset_extend_iter_map(raw: &str) -> String {
    let mut values = HashSet::new();
    values.extend([raw.to_string()]);
    values
        .iter()
        .map(|value| HashsetExtendIterMapPayload::new(value).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_hashset_extend_iter_map(raw: &str) -> String {
    HashsetExtendIterMapPayload::new(raw).unused_label()
}
