pub fn selected_map_while_report(raw: &str) -> String {
    while_model::selected_map_while(raw)
}

pub fn dead_live_map_while_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
