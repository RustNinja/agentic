use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_vec_tuple_report(raw: &str) -> String {
    collect_vec_tuple_api::selected_collect_vec_tuple_report(raw)
}

pub fn dead_collect_vec_tuple_report(raw: &str) -> String {
    collect_vec_tuple_api::dead_collect_vec_tuple_report(raw)
}
