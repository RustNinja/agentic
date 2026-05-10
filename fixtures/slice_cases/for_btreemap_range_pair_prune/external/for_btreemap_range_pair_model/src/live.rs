use std::collections::BTreeMap;

pub fn selected_for_btreemap_range_pair(raw: &str) -> String {
    let items = for_btreemap_range_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.range(0..=3) {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForBtreemapRangePairPayload {
    value: String,
}

impl ForBtreemapRangePairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_btreemap_range_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_btreemap_range_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-btreemap-range-pair:{}", self.value)
    }
}

fn for_btreemap_range_pair_items(raw: &str) -> BTreeMap<i32, ForBtreemapRangePairPayload> {
    BTreeMap::from([
        (1, ForBtreemapRangePairPayload::new(raw)),
        (2, ForBtreemapRangePairPayload::new("tail")),
    ])
}

pub fn dead_live_for_btreemap_range_pair(raw: &str) -> String {
    ForBtreemapRangePairPayload::new(raw).unused_label()
}
