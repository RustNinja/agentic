pub fn selected_take_while_report(raw: &str) -> String {
    take_model::selected_take_while(raw)
}

pub fn dead_live_take_while_report(raw: &str) -> String {
    format!("dead-live-take:{raw}")
}
