use std::collections::BTreeSet;
pub struct BtreesetRangeRevMapPayload {
    value: String,
}

impl BtreesetRangeRevMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-range-rev-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreeset-range-rev-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreeset-range-rev-map:{}", self.value)
    }
}

pub fn selected_btreeset_range_rev_map(raw: &str) -> String {
    let mut values = BTreeSet::new();
    values.insert(raw.to_string());
    values.insert("zz".to_string());
    values
        .range(..=raw.to_string())
        .rev()
        .map(|value| BtreesetRangeRevMapPayload::new(value).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreeset_range_rev_map(raw: &str) -> String {
    BtreesetRangeRevMapPayload::new(raw).unused_label()
}
