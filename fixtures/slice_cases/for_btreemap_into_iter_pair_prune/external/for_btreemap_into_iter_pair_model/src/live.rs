use std::collections::BTreeMap;

pub fn selected_for_btreemap_into_iter_pair(raw: &str) -> String {
    let items = for_btreemap_into_iter_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForBtreemapIntoIterPairPayload {
    value: String,
}

impl ForBtreemapIntoIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_btreemap_into_iter_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_btreemap_into_iter_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-btreemap-into-iter-pair:{}", self.value)
    }
}

fn for_btreemap_into_iter_pair_items(raw: &str) -> BTreeMap<i32, ForBtreemapIntoIterPairPayload> {
    BTreeMap::from([
        (1, ForBtreemapIntoIterPairPayload::new(raw)),
        (2, ForBtreemapIntoIterPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_btreemap_into_iter_pair(raw: &str) -> String {
    ForBtreemapIntoIterPairPayload::new(raw).unused_label()
}
