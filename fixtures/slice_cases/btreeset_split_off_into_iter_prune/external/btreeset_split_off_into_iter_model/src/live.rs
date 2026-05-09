use std::collections::BTreeSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreesetSplitOffIntoIterPayload {
    value: String,
}

impl BtreesetSplitOffIntoIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-split-off-into-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("btreeset-split-off-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-split-off-into-iter:{}", self.value)
    }
}

fn btreeset_split_off_into_iter_payloads(raw: &str) -> BTreeSet<BtreesetSplitOffIntoIterPayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetSplitOffIntoIterPayload::new(raw));
    payloads.insert(BtreesetSplitOffIntoIterPayload::new("tail"));
    payloads
}

pub fn selected_btreeset_split_off_into_iter(raw: &str) -> String {
    let mut payloads = btreeset_split_off_into_iter_payloads(raw);
    payloads
        .split_off(&BtreesetSplitOffIntoIterPayload::new("tail"))
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreeset-split-off-into-iter:missing".to_string())
}

pub fn dead_live_btreeset_split_off_into_iter(raw: &str) -> String {
    BtreesetSplitOffIntoIterPayload::new(raw).dead_method()
}
