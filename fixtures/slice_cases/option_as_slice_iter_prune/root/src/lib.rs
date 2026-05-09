use opensourced::opensourced;

#[opensourced]
pub fn selected_option_as_slice_iter_report(raw: &str) -> String {
    option_as_slice_iter_api::selected_option_as_slice_iter_report(raw)
}

pub fn dead_option_as_slice_iter_report(raw: &str) -> String {
    option_as_slice_iter_api::dead_option_as_slice_iter_report(raw)
}
