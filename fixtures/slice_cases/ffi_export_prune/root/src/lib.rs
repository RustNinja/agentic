use opensourced::opensourced;

#[opensourced]
pub fn selected_ffi_report(raw: &str) -> usize {
    ffi_api::selected_ffi_report(raw)
}

pub fn dead_ffi_report(raw: &str) -> usize {
    ffi_api::dead_ffi_report(raw)
}
