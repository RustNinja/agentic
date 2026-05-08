pub struct FfiState {
    len: usize,
}

impl FfiState {
    pub fn from_raw(_ptr: *const u8, len: usize) -> Self {
        Self { len }
    }

    pub fn score(&self) -> usize {
        self.len + 1
    }

    pub fn dead_method(&self) -> usize {
        self.len + 99
    }
}

pub fn selected_ffi_len(ptr: *const u8, len: usize) -> usize {
    FfiState::from_raw(ptr, len).score()
}

pub fn dead_live_ffi_len(ptr: *const u8, len: usize) -> usize {
    FfiState::from_raw(ptr, len).dead_method()
}
