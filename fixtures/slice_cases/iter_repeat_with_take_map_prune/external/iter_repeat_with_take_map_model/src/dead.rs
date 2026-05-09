pub struct DeadIterRepeatWithTakeMapItem {
    value: String,
}

impl DeadIterRepeatWithTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-repeat-with-take-map:{}", self.value)
    }
}

pub fn dead_iter_repeat_with_take_map(raw: &str) -> String {
    DeadIterRepeatWithTakeMapItem::new(raw).dead_method()
}
