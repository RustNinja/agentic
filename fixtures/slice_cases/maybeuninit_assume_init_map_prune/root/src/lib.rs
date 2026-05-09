use opensourced::opensourced;

#[opensourced]
pub fn selected_maybeuninit_assume_init_map_report(raw: &str) -> String {
    maybeuninit_assume_init_map_api::selected_maybeuninit_assume_init_map_report(raw)
}

pub fn dead_maybeuninit_assume_init_map_report(raw: &str) -> String {
    maybeuninit_assume_init_map_api::dead_maybeuninit_assume_init_map_report(raw)
}
