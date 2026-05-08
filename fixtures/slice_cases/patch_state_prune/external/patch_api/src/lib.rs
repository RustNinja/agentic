mod dead;
mod live;

pub use live::selected_patch_report;

pub fn dead_patch_report(raw: &str) -> String {
    dead::dead_patch_report(raw)
}
