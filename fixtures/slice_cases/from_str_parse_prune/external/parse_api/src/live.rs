pub fn selected_parse_report(raw: &str) -> String {
    parse_model::selected_parse(raw)
}

pub fn dead_live_parse_report(raw: &str) -> String {
    format!("dead-live-parse:{raw}")
}
