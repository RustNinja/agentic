pub struct DeadControlFlowContinueMatchMapItem {
    value: String,
}

impl DeadControlFlowContinueMatchMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-control-flow-continue-match-map:{}", self.value)
    }
}

pub fn dead_control_flow_continue_match_map(raw: &str) -> String {
    DeadControlFlowContinueMatchMapItem::new(raw).render()
}
