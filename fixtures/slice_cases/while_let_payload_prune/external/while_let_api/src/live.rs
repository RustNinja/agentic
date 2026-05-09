pub fn selected_while_let_report(raw: &str) -> String {
    while_let_model::selected_while_let(raw)
}

pub fn dead_live_while_let_report(raw: &str) -> String {
    format!("dead-live-while-let:{raw}")
}
