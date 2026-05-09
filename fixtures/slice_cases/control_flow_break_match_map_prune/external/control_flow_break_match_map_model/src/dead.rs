pub struct DeadControlFlowBreakMatchMapItem {
    value: String,
}

impl DeadControlFlowBreakMatchMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-control-flow-break-match-map:{}", self.value)
    }
}

pub fn dead_control_flow_break_match_map(raw: &str) -> String {
    DeadControlFlowBreakMatchMapItem::new(raw).render()
}
