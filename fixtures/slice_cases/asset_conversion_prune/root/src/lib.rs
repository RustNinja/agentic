use opensourced::opensourced;

#[opensourced]
pub fn selected_asset_report(raw: &str) -> String {
    asset_api::selected_asset_report(raw)
}

pub fn dead_asset_report(raw: &str) -> String {
    asset_api::dead_asset_report(raw)
}
