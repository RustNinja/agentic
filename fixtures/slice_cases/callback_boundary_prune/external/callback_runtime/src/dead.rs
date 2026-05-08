pub struct DeadInput {
    value: u32,
}

impl DeadInput {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

pub struct DeadCallback;

impl DeadCallback {
    pub fn score_dead(&self, input: DeadInput) -> u32 {
        input.value + 99
    }
}

pub fn dead_callback_fn(value: u32) -> u32 {
    value + 99
}
