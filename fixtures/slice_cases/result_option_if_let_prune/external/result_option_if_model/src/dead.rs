pub struct DeadResultOptionIfItem {
    value: String,
}

impl DeadResultOptionIfItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-option-if:{}", self.value)
    }
}

pub fn dead_result_option_if(raw: &str) -> String {
    DeadResultOptionIfItem::new(raw).dead_method()
}
