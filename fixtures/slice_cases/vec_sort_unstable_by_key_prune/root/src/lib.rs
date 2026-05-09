use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_sort_unstable_by_key_report(raw: &str) -> String {
    vec_sort_unstable_by_key_api::selected_vec_sort_unstable_by_key_report(raw)
}

pub fn dead_vec_sort_unstable_by_key_report(raw: &str) -> String {
    vec_sort_unstable_by_key_api::dead_vec_sort_unstable_by_key_report(raw)
}
