use opensourced::opensourced;

#[opensourced]
pub fn selected_mutex_report(raw: &str) -> String {
    mutex_api::selected_mutex_report(raw)
}

pub fn dead_mutex_report(raw: &str) -> String {
    mutex_api::dead_mutex_report(raw)
}
