use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_shrink_to_fit_iter_map_report(raw: &str) -> String {
    hashset_shrink_to_fit_iter_map_api::selected_hashset_shrink_to_fit_iter_map_report(raw)
}

pub fn dead_hashset_shrink_to_fit_iter_map_report(raw: &str) -> String {
    hashset_shrink_to_fit_iter_map_api::dead_hashset_shrink_to_fit_iter_map_report(raw)
}
