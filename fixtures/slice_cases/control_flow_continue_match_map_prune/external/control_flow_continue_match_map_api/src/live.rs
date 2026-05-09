pub fn selected_control_flow_continue_match_map_report(raw: &str) -> String {
    control_flow_continue_match_map_model::selected_control_flow_continue_match_map(raw)
}

pub fn dead_live_control_flow_continue_match_map_report(raw: &str) -> String {
    format!("dead-live-control-flow-continue-match-map:{raw}")
}
