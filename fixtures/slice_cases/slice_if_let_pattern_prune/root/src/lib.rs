use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_if_let_pattern_report(raw: &str) -> String {
    slice_if_let_pattern_api::selected_slice_if_let_pattern_report(raw)
}

pub fn dead_slice_if_let_pattern_report(raw: &str) -> String {
    slice_if_let_pattern_api::dead_slice_if_let_pattern_report(raw)
}
