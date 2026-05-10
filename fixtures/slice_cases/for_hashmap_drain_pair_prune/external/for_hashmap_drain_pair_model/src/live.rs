use std::collections::HashMap;

pub fn selected_for_hashmap_drain_pair(raw: &str) -> String {
    let mut items = for_hashmap_drain_pair_items(raw);
    let mut rendered = Vec::new();
    for (_key, payload) in items.drain() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForHashmapDrainPairPayload {
    value: String,
}

impl ForHashmapDrainPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_hashmap_drain_pair:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_hashmap_drain_pair:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-hashmap-drain-pair:{}", self.value)
    }
}

fn for_hashmap_drain_pair_items(raw: &str) -> HashMap<String, ForHashmapDrainPairPayload> {
    HashMap::from([
        (String::from("live"), ForHashmapDrainPairPayload::new(raw)),
        (String::from("tail"), ForHashmapDrainPairPayload::new("tail")),
    ])
}

pub fn dead_live_for_hashmap_drain_pair(raw: &str) -> String {
    ForHashmapDrainPairPayload::new(raw).unused_label()
}
