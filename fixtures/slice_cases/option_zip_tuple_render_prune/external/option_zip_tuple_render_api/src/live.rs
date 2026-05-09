pub fn selected_option_zip_tuple_render_report(raw: &str) -> String {
    option_zip_tuple_render_model::selected_option_zip_tuple_render(raw)
}

pub fn dead_live_option_zip_tuple_render_report(raw: &str) -> String {
    format!("dead-option-zip-tuple-render-live-report:{raw}")
}
