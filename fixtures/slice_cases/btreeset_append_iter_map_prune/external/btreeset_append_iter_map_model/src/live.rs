use std::collections::BTreeSet;
pub struct BtreesetAppendIterMapPayload {
    value: String,
}

impl BtreesetAppendIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-append-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreeset-append-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreeset-append-iter-map:{}", self.value)
    }
}

pub fn selected_btreeset_append_iter_map(raw: &str) -> String {
    let mut values = BTreeSet::new();
    values.insert("old".to_string());
    let mut extras = BTreeSet::new();
    extras.insert(raw.to_string());
    values.append(&mut extras);
    values
        .iter()
        .map(|value| BtreesetAppendIterMapPayload::new(value).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_btreeset_append_iter_map(raw: &str) -> String {
    BtreesetAppendIterMapPayload::new(raw).unused_label()
}
