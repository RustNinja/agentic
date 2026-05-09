pub struct DeadIterFromFnMapItem {
    value: String,
}

impl DeadIterFromFnMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iter-from-fn-map:{}", self.value)
    }
}

pub fn dead_iter_from_fn_map(raw: &str) -> String {
    DeadIterFromFnMapItem::new(raw).render()
}
