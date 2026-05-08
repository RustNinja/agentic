pub fn selected_try_fold_report(raw: &str) -> String {
    try_fold_model::selected_try_fold(raw)
}

pub fn dead_live_try_fold_report(raw: &str) -> String {
    format!("dead-live-try-fold:{raw}")
}
