use opensourced::opensourced;

#[opensourced]
pub fn selected_result_as_ref_map_report(raw: &str) -> String {
    result_as_ref_api::selected_result_as_ref_map_report(raw)
}

pub fn dead_result_as_ref_map_report(raw: &str) -> String {
    result_as_ref_api::dead_result_as_ref_map_report(raw)
}
