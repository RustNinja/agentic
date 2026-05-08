pub fn selected_array_report(raw: &str) -> String {
    array_model::selected_array(raw).render()
}

pub fn dead_live_array_report(raw: &str) -> String {
    format!("dead-live-array:{raw}")
}
