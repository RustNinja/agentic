use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_annotated_result_vec_report(raw: &str) -> String {
    collect_annotated_result_vec_api::selected_collect_annotated_result_vec_report(raw)
}

pub fn dead_collect_annotated_result_vec_report(raw: &str) -> String {
    collect_annotated_result_vec_api::dead_live_collect_annotated_result_vec(raw)
}
