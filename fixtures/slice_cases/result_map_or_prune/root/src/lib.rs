use opensourced::opensourced;

#[opensourced]
pub fn selected_result_map_or_value_report(raw: &str) -> String {
    result_map_or_value_api::selected_result_map_or_value_report(raw)
}

pub fn dead_result_map_or_value_report(raw: &str) -> String {
    result_map_or_value_api::dead_result_map_or_value_report(raw)
}
