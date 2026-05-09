use opensourced::opensourced;

#[opensourced]
pub fn selected_result_map_or_report(raw: &str) -> String {
    result_map_or_api::selected_result_map_or_report(raw)
}

pub fn dead_result_map_or_report(raw: &str) -> String {
    result_map_or_api::dead_result_map_or_report(raw)
}
