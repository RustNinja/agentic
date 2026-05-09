pub struct DeadArcMakeMutMapItem {
    value: String,
}

impl DeadArcMakeMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-arc-make-mut-map:{}", self.value)
    }
}

pub fn dead_arc_make_mut_map(raw: &str) -> String {
    DeadArcMakeMutMapItem::new(raw).render()
}
