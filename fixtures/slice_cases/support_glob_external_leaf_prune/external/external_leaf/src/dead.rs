pub struct DeadLeaf {
    pub label: String,
}

pub fn dead_leaf(raw: &str) -> DeadLeaf {
    DeadLeaf {
        label: raw.to_string(),
    }
}
