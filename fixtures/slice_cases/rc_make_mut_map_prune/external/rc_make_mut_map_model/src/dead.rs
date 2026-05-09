pub struct DeadRcMakeMutMapItem {
    value: String,
}

impl DeadRcMakeMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-rc-make-mut-map:{}", self.value)
    }
}

pub fn dead_rc_make_mut_map(raw: &str) -> String {
    DeadRcMakeMutMapItem::new(raw).render()
}
