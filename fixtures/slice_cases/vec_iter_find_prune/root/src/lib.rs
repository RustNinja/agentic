use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_iter_find_report(raw: &str) -> String {
    vec_iter_find_api::selected_vec_iter_find_report(raw)
}

pub fn dead_vec_iter_find_report(raw: &str) -> String {
    vec_iter_find_api::dead_vec_iter_find_report(raw)
}
