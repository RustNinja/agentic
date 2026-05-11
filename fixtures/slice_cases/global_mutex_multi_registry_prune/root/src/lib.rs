use opensourced::opensourced;

#[opensourced]
pub fn selected_registry_lookup(raw: &str) -> String {
    registry_api::selected_registry_lookup(raw)
}

pub fn dead_registry_lookup(raw: &str) -> String {
    registry_api::dead_registry_lookup(raw)
}
