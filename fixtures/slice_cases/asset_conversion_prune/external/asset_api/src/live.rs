use asset_codec::{convert_selected_asset, AssetError};

pub fn selected_asset_report(raw: &str) -> String {
    convert_selected_asset(raw)
        .map(|report| report.render())
        .unwrap_or_else(AssetError::render)
}
