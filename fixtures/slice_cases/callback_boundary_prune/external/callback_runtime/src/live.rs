pub trait Callback {
    fn score(&self, input: &CallbackInput) -> u32;
}

pub type CallbackFn = fn(u32) -> u32;

pub struct CallbackInput {
    value: u32,
}

impl CallbackInput {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

pub fn apply_callback(handler: &dyn Callback, callback: CallbackFn, input: CallbackInput) -> u32 {
    callback(handler.score(&input))
}

pub fn bump_callback(value: u32) -> u32 {
    value + 1
}

pub fn unused_callback_fn(value: u32) -> u32 {
    value + 99
}
