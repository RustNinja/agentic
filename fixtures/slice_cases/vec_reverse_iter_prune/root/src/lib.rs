use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_reverse_iter_report(raw: &str) -> String {
    vec_reverse_iter_api::selected_vec_reverse_iter_report(raw)
}

pub fn dead_vec_reverse_iter_report(raw: &str) -> String {
    vec_reverse_iter_api::dead_vec_reverse_iter_report(raw)
}
