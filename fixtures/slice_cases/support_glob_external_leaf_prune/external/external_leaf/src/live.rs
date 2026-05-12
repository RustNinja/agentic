pub struct LeafRecord {
    pub label: String,
}

pub fn dead_live_leaf(raw: &str) -> String {
    format!("dead-live:{raw}")
}
