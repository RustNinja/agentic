use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_annotated_vec_tuple_report(raw: &str) -> String {
    collect_annotated_vec_tuple_api::selected_collect_annotated_vec_tuple_report(raw)
}

pub fn dead_collect_annotated_vec_tuple_report(raw: &str) -> String {
    collect_annotated_vec_tuple_api::dead_live_collect_annotated_vec_tuple(raw)
}
