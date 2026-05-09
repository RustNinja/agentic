use opensourced::opensourced;

#[opensourced]
pub fn selected_control_flow_break_match_map_report(raw: &str) -> String {
    control_flow_break_match_map_api::selected_control_flow_break_match_map_report(raw)
}

pub fn dead_control_flow_break_match_map_report(raw: &str) -> String {
    control_flow_break_match_map_api::dead_control_flow_break_match_map_report(raw)
}
