use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_split_report(raw: &str) -> String {
    slice_split_api::selected_slice_split_report(raw)
}

pub fn dead_slice_split_report(raw: &str) -> String {
    slice_split_api::dead_slice_split_report(raw)
}
