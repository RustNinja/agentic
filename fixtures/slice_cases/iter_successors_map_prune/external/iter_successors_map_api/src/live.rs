pub fn selected_iter_successors_map_report(raw: &str) -> String {
    iter_successors_map_model::selected_iter_successors_map(raw)
}

pub fn dead_live_iter_successors_map_report(raw: &str) -> String {
    format!("dead-live-iter-successors-map:{raw}")
}
