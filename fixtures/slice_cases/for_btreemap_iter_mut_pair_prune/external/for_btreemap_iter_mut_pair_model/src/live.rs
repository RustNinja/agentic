use std::collections::BTreeMap;

pub fn selected_for_btreemap_iter_mut_pair(raw: &str) -> String {
    let mut items = for_btreemap_iter_mut_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.iter_mut() {
        rendered.push(payload.bump_and_render());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForBtreemapIterMutPairPayload {
    value: String,
}

impl ForBtreemapIterMutPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_btreemap_iter_mut_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_btreemap_iter_mut_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-btreemap-iter-mut-pair:{}", self.value)
    }
}

fn for_btreemap_iter_mut_pair_items(raw: &str) -> BTreeMap<i32, ForBtreemapIterMutPairPayload> {
    BTreeMap::from([
        (1, ForBtreemapIterMutPairPayload::new(raw)),
        (2, ForBtreemapIterMutPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_btreemap_iter_mut_pair(raw: &str) -> String {
    ForBtreemapIterMutPairPayload::new(raw).unused_label()
}
