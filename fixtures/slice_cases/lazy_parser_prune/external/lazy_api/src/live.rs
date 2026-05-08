pub fn selected_lazy_report(raw: &str) -> String {
    lazy_model::selected_lazy(raw)
}

pub fn dead_live_lazy_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
