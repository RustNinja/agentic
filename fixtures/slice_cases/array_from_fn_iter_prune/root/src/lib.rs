use opensourced::opensourced;

#[opensourced]
pub fn selected_array_from_fn_iter_report(raw: &str) -> String {
    array_from_fn_iter_api::selected_array_from_fn_iter_report(raw)
}

pub fn dead_array_from_fn_iter_report(raw: &str) -> String {
    array_from_fn_iter_api::dead_array_from_fn_iter_report(raw)
}
