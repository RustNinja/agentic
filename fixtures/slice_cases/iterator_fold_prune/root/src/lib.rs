use opensourced::opensourced;

#[opensourced]
pub fn selected_fold_report(raw: &str) -> String {
    fold_api::selected_fold_report(raw)
}

pub fn dead_fold_report(raw: &str) -> String {
    fold_api::dead_fold_report(raw)
}
