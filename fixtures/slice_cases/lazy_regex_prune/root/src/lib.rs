use opensourced::opensourced;

#[opensourced]
pub fn selected_route_report(raw: &str) -> String {
    regex_api::selected_route_report(raw)
}

pub fn dead_route_report(raw: &str) -> String {
    regex_api::dead_route_report(raw)
}
