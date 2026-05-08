use opensourced::opensourced;

#[opensourced]
pub fn selected_partition_report(raw: &str) -> String {
    partition_api::selected_partition_report(raw)
}

pub fn dead_partition_report(raw: &str) -> String {
    partition_api::dead_partition_report(raw)
}
