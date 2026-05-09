pub struct DeadIterSuccessorsMapItem {
    value: String,
}

impl DeadIterSuccessorsMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iter-successors-map:{}", self.value)
    }
}

pub fn dead_iter_successors_map(raw: &str) -> String {
    DeadIterSuccessorsMapItem::new(raw).render()
}
