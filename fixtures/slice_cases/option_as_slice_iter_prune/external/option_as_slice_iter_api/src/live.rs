pub fn selected_option_as_slice_iter_report(raw: &str) -> String {
    option_as_slice_iter_model::selected_option_as_slice_iter(raw)
}

pub fn dead_live_option_as_slice_iter_report(raw: &str) -> String {
    format!("dead-live-option-as-slice-iter:{raw}")
}
