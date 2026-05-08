use opensourced::opensourced;

#[opensourced]
pub fn selected_try_fold_report(raw: &str) -> String {
    try_fold_api::selected_try_fold_report(raw)
}

pub fn dead_try_fold_report(raw: &str) -> String {
    try_fold_api::dead_try_fold_report(raw)
}
