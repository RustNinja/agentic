use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_split_off_into_iter_report(raw: &str) -> String {
    vec_split_off_into_iter_api::selected_vec_split_off_into_iter_report(raw)
}

pub fn dead_vec_split_off_into_iter_report(raw: &str) -> String {
    vec_split_off_into_iter_api::dead_vec_split_off_into_iter_report(raw)
}
