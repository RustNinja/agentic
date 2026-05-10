use std::collections::BTreeMap;

pub fn selected_for_btreemap_range_mut_pair(raw: &str) -> String {
    let mut items = for_btreemap_range_mut_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.range_mut(0..=3) {
        rendered.push(payload.bump_and_render());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForBtreemapRangeMutPairPayload {
    value: String,
}

impl ForBtreemapRangeMutPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_btreemap_range_mut_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_btreemap_range_mut_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-btreemap-range-mut-pair:{}", self.value)
    }
}

fn for_btreemap_range_mut_pair_items(raw: &str) -> BTreeMap<i32, ForBtreemapRangeMutPairPayload> {
    BTreeMap::from([
        (1, ForBtreemapRangeMutPairPayload::new(raw)),
        (2, ForBtreemapRangeMutPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_btreemap_range_mut_pair(raw: &str) -> String {
    ForBtreemapRangeMutPairPayload::new(raw).unused_label()
}
