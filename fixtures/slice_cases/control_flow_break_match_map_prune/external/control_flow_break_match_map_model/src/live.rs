use std::ops::ControlFlow;

#[derive(Clone)]
pub struct ControlFlowBreakMatchMapPayload {
    value: String,
}

impl ControlFlowBreakMatchMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("control-flow-break-match-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("control-flow-break-match-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-control-flow-break-match-map:{}", self.value)
    }
}

pub fn selected_control_flow_break_match_map(raw: &str) -> String {
    let flow: ControlFlow<ControlFlowBreakMatchMapPayload, ()> =
        ControlFlow::Break(ControlFlowBreakMatchMapPayload::new(raw));
    match flow {
        ControlFlow::Break(payload) => payload.render_label(),
        ControlFlow::Continue(()) => "control-flow-break-match-map:continue".to_string(),
    }
}

pub fn dead_live_control_flow_break_match_map(raw: &str) -> String {
    ControlFlowBreakMatchMapPayload::new(raw).dead_method()
}
