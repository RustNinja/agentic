pub fn selected_route_report(raw: &str) -> String {
    parser_support::selected_route_id(raw)
}

pub fn dead_live_route_report(raw: &str) -> String {
    format!("dead-live-route:{raw}")
}
