pub fn selected_for_each_report(raw: &str) -> String {
    foreach_model::selected_for_each(raw)
}

pub fn dead_live_for_each_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
