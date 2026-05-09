use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_as_mut_slice_iter_mut_report(raw: &str) -> String {
    vec_as_mut_slice_iter_mut_api::selected_vec_as_mut_slice_iter_mut_report(raw)
}

pub fn dead_vec_as_mut_slice_iter_mut_report(raw: &str) -> String {
    vec_as_mut_slice_iter_mut_api::dead_vec_as_mut_slice_iter_mut_report(raw)
}
