pub struct DeadMemTakeMapItem {
    value: String,
}

impl DeadMemTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-mem-take-map:{}", self.value)
    }
}

pub fn dead_mem_take_map(raw: &str) -> String {
    DeadMemTakeMapItem::new(raw).render()
}
