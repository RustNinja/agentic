pub fn selected_tuple_partition_report(raw: &str) -> String {
    tuple_partition_model::selected_tuple_partition(raw)
}

pub fn dead_live_tuple_partition_report(raw: &str) -> String {
    format!("dead-live-tuple-partition:{raw}")
}
