use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_partition_report(raw: &str) -> String {
    tuple_partition_api::selected_tuple_partition_report(raw)
}

pub fn dead_tuple_partition_report(raw: &str) -> String {
    tuple_partition_api::dead_tuple_partition_report(raw)
}
