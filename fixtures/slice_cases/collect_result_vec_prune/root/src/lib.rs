use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_result_vec_report(raw: &str) -> String {
    collect_result_vec_api::selected_collect_result_vec_report(raw)
}

pub fn dead_collect_result_vec_report(raw: &str) -> String {
    collect_result_vec_api::dead_collect_result_vec_report(raw)
}
