use opensourced::opensourced;

#[opensourced]
pub fn selected_mem_take_map_report(raw: &str) -> String {
    mem_take_map_api::selected_mem_take_map_report(raw)
}

pub fn dead_mem_take_map_report(raw: &str) -> String {
    mem_take_map_api::dead_mem_take_map_report(raw)
}
