pub fn selected_partition_report(raw: &str) -> String {
    partition_model::selected_partition(raw)
}

pub fn dead_live_partition_report(raw: &str) -> String {
    format!("dead-live-partition:{raw}")
}
