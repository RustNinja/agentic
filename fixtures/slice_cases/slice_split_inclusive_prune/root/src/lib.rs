use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_split_inclusive_report(raw: &str) -> String {
    slice_split_inclusive_api::selected_slice_split_inclusive_report(raw)
}

pub fn dead_slice_split_inclusive_report(raw: &str) -> String {
    slice_split_inclusive_api::dead_slice_split_inclusive_report(raw)
}
