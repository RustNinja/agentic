pub struct DeadFfiState {
    len: usize,
}

pub fn dead_ffi_len(_ptr: *const u8, len: usize) -> usize {
    DeadFfiState { len }.len + 1000
}
