use opensourced::opensourced;

#[opensourced]
pub fn selected_map_while_report(raw: &str) -> String {
    while_api::selected_map_while_report(raw)
}

pub fn dead_map_while_report(raw: &str) -> String {
    while_api::dead_map_while_report(raw)
}
