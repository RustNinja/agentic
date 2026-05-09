use opensourced::opensourced;

#[opensourced]
pub fn selected_phantomdata_surface_map_report(raw: &str) -> String {
    phantomdata_surface_api::selected_phantomdata_surface_map_report(raw)
}

pub fn dead_phantomdata_surface_map_report(raw: &str) -> String {
    phantomdata_surface_api::dead_phantomdata_surface_map_report(raw)
}
