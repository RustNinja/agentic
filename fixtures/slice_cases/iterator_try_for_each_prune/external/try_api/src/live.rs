pub fn selected_try_for_each_report(raw: &str) -> String {
    try_model::selected_try_for_each(raw)
}

pub fn dead_live_try_for_each_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
