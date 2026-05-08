pub fn selected_macro_report(raw: &str) -> String {
    macro_support::selected_generated(raw)
}

pub fn dead_live_macro_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
