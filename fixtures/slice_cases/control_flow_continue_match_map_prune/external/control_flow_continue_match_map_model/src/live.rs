use std::ops::ControlFlow;

#[derive(Clone)]
pub struct ControlFlowContinueMatchMapPayload {
    value: String,
}

impl ControlFlowContinueMatchMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("control-flow-continue-match-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("control-flow-continue-match-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-control-flow-continue-match-map:{}", self.value)
    }
}

pub fn selected_control_flow_continue_match_map(raw: &str) -> String {
    let flow: ControlFlow<(), ControlFlowContinueMatchMapPayload> =
        ControlFlow::Continue(ControlFlowContinueMatchMapPayload::new(raw));
    match flow {
        ControlFlow::Continue(payload) => payload.render_label(),
        ControlFlow::Break(()) => "control-flow-continue-match-map:break".to_string(),
    }
}

pub fn dead_live_control_flow_continue_match_map(raw: &str) -> String {
    ControlFlowContinueMatchMapPayload::new(raw).dead_method()
}
