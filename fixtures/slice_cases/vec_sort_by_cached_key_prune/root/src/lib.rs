use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_sort_by_cached_key_report(raw: &str) -> String {
    vec_sort_by_cached_key_api::selected_vec_sort_by_cached_key_report(raw)
}

pub fn dead_vec_sort_by_cached_key_report(raw: &str) -> String {
    vec_sort_by_cached_key_api::dead_vec_sort_by_cached_key_report(raw)
}
