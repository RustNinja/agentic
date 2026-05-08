pub fn dead_ffi_report(raw: &str) -> usize {
    ffi_support::dead_ffi_len(raw.as_ptr(), raw.len())
}
