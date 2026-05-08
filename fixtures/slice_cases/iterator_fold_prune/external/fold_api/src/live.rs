pub fn selected_fold_report(raw: &str) -> String {
    fold_model::selected_fold(raw)
}

pub fn dead_live_fold_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
