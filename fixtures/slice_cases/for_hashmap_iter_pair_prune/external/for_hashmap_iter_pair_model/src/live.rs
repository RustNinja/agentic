use std::collections::HashMap;

pub fn selected_for_hashmap_iter_pair(raw: &str) -> String {
    let items = for_hashmap_iter_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.iter() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForHashmapIterPairPayload {
    value: String,
}

impl ForHashmapIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_hashmap_iter_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_hashmap_iter_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-hashmap-iter-pair:{}", self.value)
    }
}

fn for_hashmap_iter_pair_items(raw: &str) -> HashMap<String, ForHashmapIterPairPayload> {
    HashMap::from([
        (String::from("live"), ForHashmapIterPairPayload::new(raw)),
        (String::from("tail"), ForHashmapIterPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_hashmap_iter_pair(raw: &str) -> String {
    ForHashmapIterPairPayload::new(raw).unused_label()
}
