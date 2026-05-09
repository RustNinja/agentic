pub struct DeadArrayFromFnIterItem {
    value: String,
}

impl DeadArrayFromFnIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-array-from-fn-iter:{}", self.value)
    }
}

pub fn dead_array_from_fn_iter(raw: &str) -> String {
    DeadArrayFromFnIterItem::new(raw).render()
}
