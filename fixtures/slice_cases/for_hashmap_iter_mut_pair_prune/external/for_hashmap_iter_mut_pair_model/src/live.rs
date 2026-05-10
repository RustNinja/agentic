use std::collections::HashMap;

pub fn selected_for_hashmap_iter_mut_pair(raw: &str) -> String {
    let mut items = for_hashmap_iter_mut_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.iter_mut() {
        rendered.push(payload.bump_and_render());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForHashmapIterMutPairPayload {
    value: String,
}

impl ForHashmapIterMutPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_hashmap_iter_mut_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_hashmap_iter_mut_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-hashmap-iter-mut-pair:{}", self.value)
    }
}

fn for_hashmap_iter_mut_pair_items(raw: &str) -> HashMap<String, ForHashmapIterMutPairPayload> {
    HashMap::from([
        (String::from("live"), ForHashmapIterMutPairPayload::new(raw)),
        (String::from("tail"), ForHashmapIterMutPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_hashmap_iter_mut_pair(raw: &str) -> String {
    ForHashmapIterMutPairPayload::new(raw).unused_label()
}
