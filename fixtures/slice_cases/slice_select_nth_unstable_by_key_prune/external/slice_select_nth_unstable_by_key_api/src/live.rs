pub fn selected_slice_select_nth_unstable_by_key_report(raw: &str) -> String {
    slice_select_nth_unstable_by_key_model::selected_slice_select_nth_unstable_by_key(raw)
}

pub fn dead_live_slice_select_nth_unstable_by_key_report(raw: &str) -> String {
    format!("dead-slice-select-nth-unstable-by-key-live-report:{raw}")
}
