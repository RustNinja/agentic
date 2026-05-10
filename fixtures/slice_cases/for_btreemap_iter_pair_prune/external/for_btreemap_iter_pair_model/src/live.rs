use std::collections::BTreeMap;

pub fn selected_for_btreemap_iter_pair(raw: &str) -> String {
    let items = for_btreemap_iter_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.iter() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForBtreemapIterPairPayload {
    value: String,
}

impl ForBtreemapIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_btreemap_iter_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_btreemap_iter_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-btreemap-iter-pair:{}", self.value)
    }
}

fn for_btreemap_iter_pair_items(raw: &str) -> BTreeMap<i32, ForBtreemapIterPairPayload> {
    BTreeMap::from([
        (1, ForBtreemapIterPairPayload::new(raw)),
        (2, ForBtreemapIterPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_btreemap_iter_pair(raw: &str) -> String {
    ForBtreemapIterPairPayload::new(raw).unused_label()
}
