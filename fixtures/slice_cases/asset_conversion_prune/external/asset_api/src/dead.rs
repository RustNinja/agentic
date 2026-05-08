use asset_codec::{build_dead_asset, DeadAssetReport};

pub fn dead_asset_report(raw: &str) -> String {
    let report: DeadAssetReport = build_dead_asset(raw);
    report.render_dead()
}
