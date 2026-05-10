use std::collections::BTreeSet;
pub struct BtreesetSplitOffIterMapPayload {
    value: String,
}

impl BtreesetSplitOffIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-split-off-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreeset-split-off-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreeset-split-off-iter-map:{}", self.value)
    }
}

pub fn selected_btreeset_split_off_iter_map(raw: &str) -> String {
    let mut values = BTreeSet::new();
    values.insert("head".to_string());
    values.insert(raw.to_string());
    let tail = values.split_off(raw);
    tail.iter()
        .map(|value| BtreesetSplitOffIterMapPayload::new(value).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_btreeset_split_off_iter_map(raw: &str) -> String {
    BtreesetSplitOffIterMapPayload::new(raw).unused_label()
}
