pub fn selected_dyn_report(raw: &str) -> String {
    let reader = dyn_model::selected_reader(raw);
    dyn_model::render_reader(reader)
}

pub fn dead_live_dyn_report(raw: &str) -> String {
    format!("dead-live-dyn:{raw}")
}

