use callback_runtime::{dead_callback_fn, DeadCallback, DeadInput};

pub fn dead_callback_score(value: u32) -> u32 {
    let callback = DeadCallback;
    dead_callback_fn(callback.score_dead(DeadInput::new(value)))
}
