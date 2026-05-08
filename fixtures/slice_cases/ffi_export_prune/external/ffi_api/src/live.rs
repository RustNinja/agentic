pub fn selected_ffi_report(raw: &str) -> usize {
    selected_ffi_export(raw.as_ptr(), raw.len())
}

#[no_mangle]
pub extern "C" fn selected_ffi_export(ptr: *const u8, len: usize) -> usize {
    ffi_support::selected_ffi_len(ptr, len)
}

pub fn dead_live_ffi_report(raw: &str) -> usize {
    raw.len() + 100
}
