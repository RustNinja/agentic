use opensourced::opensourced;

#[opensourced]
pub fn selected_runtime_report(raw: &str) -> String {
    runtime_api::selected_runtime_report(raw)
}

pub fn dead_runtime_report(raw: &str) -> String {
    runtime_api::dead_runtime_report(raw)
}
