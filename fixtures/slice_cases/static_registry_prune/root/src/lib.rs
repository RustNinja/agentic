use opensourced::opensourced;

#[opensourced]
pub fn selected_registry_report(raw: &str) -> String {
    registry_api::selected_registry_report(raw)
}

pub fn dead_registry_report(raw: &str) -> String {
    registry_api::dead_registry_report(raw)
}
