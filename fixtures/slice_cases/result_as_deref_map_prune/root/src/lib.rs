use opensourced::opensourced;

#[opensourced]
pub fn selected_result_as_deref_map_report(raw: &str) -> String {
    result_as_deref_api::selected_result_as_deref_map_report(raw)
}

pub fn dead_result_as_deref_map_report(raw: &str) -> String {
    result_as_deref_api::dead_result_as_deref_map_report(raw)
}
