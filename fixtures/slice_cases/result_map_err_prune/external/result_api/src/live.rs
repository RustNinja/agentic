pub fn selected_result_report(raw: &str) -> String {
    result_model::parse_result(raw)
        .map(|value| value.render())
        .map_err(|err| err.render())
        .unwrap_or_else(|message| message)
}

pub fn dead_live_result_report(raw: &str) -> String {
    format!("dead-live-result:{raw}")
}
