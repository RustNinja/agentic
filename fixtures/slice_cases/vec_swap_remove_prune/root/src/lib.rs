use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_swap_remove_report(raw: &str) -> String {
    vec_swap_remove_api::selected_vec_swap_remove_report(raw)
}

pub fn dead_vec_swap_remove_report(raw: &str) -> String {
    vec_swap_remove_api::dead_vec_swap_remove_report(raw)
}
