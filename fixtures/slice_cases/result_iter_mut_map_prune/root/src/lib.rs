use opensourced::opensourced;

#[opensourced]
pub fn selected_result_iter_mut_map_report(raw: &str) -> String {
    result_iter_mut_map_api::selected_result_iter_mut_map_report(raw)
}

pub fn dead_result_iter_mut_map_report(raw: &str) -> String {
    result_iter_mut_map_api::dead_result_iter_mut_map_report(raw)
}
