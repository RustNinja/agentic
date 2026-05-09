use std::collections::BTreeSet;
pub struct BtreesetIterNextBackMapPayload {
    value: String,
}

impl BtreesetIterNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-iter-next-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreeset-iter-next-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreeset-iter-next-back-map:{}", self.value)
    }
}

pub fn selected_btreeset_iter_next_back_map(raw: &str) -> String {
    let mut values = BTreeSet::new();
    values.insert(raw.to_string());
    values
        .iter()
        .next_back()
        .map(|value| BtreesetIterNextBackMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreeset_iter_next_back_map(raw: &str) -> String {
    BtreesetIterNextBackMapPayload::new(raw).unused_label()
}
