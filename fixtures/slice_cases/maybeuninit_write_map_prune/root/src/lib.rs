use opensourced::opensourced;

#[opensourced]
pub fn selected_maybeuninit_write_map_report(raw: &str) -> String {
    maybeuninit_write_map_api::selected_maybeuninit_write_map_report(raw)
}

pub fn dead_maybeuninit_write_map_report(raw: &str) -> String {
    maybeuninit_write_map_api::dead_maybeuninit_write_map_report(raw)
}
