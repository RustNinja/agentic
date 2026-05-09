pub struct DeadMaybeUninitAssumeInitMapItem {
    value: String,
}

impl DeadMaybeUninitAssumeInitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-maybeuninit-assume-init-map:{}", self.value)
    }
}

pub fn dead_maybeuninit_assume_init_map(raw: &str) -> String {
    DeadMaybeUninitAssumeInitMapItem::new(raw).render()
}
