use opensourced::opensourced;

#[opensourced]
pub fn selected_cfg_report(raw: &str) -> String {
    cfg_api::selected_cfg_report(raw)
}

pub fn dead_cfg_report(raw: &str) -> String {
    cfg_api::dead_cfg_report(raw)
}

