pub struct DeadCycleNthItem {
    value: String,
}

impl DeadCycleNthItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cycle-nth:{}", self.value)
    }
}

pub fn dead_cycle_nth(raw: &str) -> String {
    DeadCycleNthItem::new(raw).dead_method()
}
