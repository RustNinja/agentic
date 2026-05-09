use opensourced::opensourced;

#[opensourced]
pub fn selected_result_as_deref_mut_map_report(raw: &str) -> String {
    result_as_deref_mut_api::selected_result_as_deref_mut_map_report(raw)
}

pub fn dead_result_as_deref_mut_map_report(raw: &str) -> String {
    result_as_deref_mut_api::dead_result_as_deref_mut_map_report(raw)
}
