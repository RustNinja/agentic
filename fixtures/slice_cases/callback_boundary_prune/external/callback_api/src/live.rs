use callback_runtime::{apply_callback, bump_callback, Callback, CallbackInput};

struct ApiCallback;

impl Callback for ApiCallback {
    fn score(&self, input: &CallbackInput) -> u32 {
        input.value() + 2
    }
}

pub fn selected_callback_score(value: u32) -> u32 {
    let callback = ApiCallback;
    apply_callback(&callback, bump_callback, CallbackInput::new(value))
}

pub fn dead_live_callback_score(value: u32) -> u32 {
    let callback = ApiCallback;
    callback.score(&CallbackInput::new(value)) + 99
}
