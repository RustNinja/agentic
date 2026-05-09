#[derive(Clone)]
pub struct CycleNthPayload {
    value: String,
}

impl CycleNthPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("cycle-nth:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cycle-nth:{}", self.value)
    }
}

fn cycle_nth_items(raw: &str) -> Vec<CycleNthPayload> {
    vec![CycleNthPayload::new(raw)]
}

pub fn selected_cycle_nth(raw: &str) -> String {
    cycle_nth_items(raw)
        .into_iter()
        .cycle()
        .nth(1)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "cycle-nth:missing".to_string())
}

pub fn dead_live_cycle_nth(raw: &str) -> String {
    CycleNthPayload::new(raw).dead_method()
}
