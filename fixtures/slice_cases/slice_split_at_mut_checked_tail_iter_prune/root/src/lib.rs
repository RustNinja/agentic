use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_split_at_mut_checked_tail_iter_report(raw: &str) -> String {
    slice_split_at_mut_checked_tail_iter_api::selected_slice_split_at_mut_checked_tail_iter_report(raw)
}

pub fn dead_slice_split_at_mut_checked_tail_iter_report(raw: &str) -> String {
    slice_split_at_mut_checked_tail_iter_api::dead_slice_split_at_mut_checked_tail_iter_report(raw)
}
