pub struct DeadArcAsRefMapItem {
    value: String,
}

impl DeadArcAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-arc-as-ref-map:{}", self.value)
    }
}

pub fn dead_arc_as_ref_map(raw: &str) -> String {
    DeadArcAsRefMapItem::new(raw).render()
}
