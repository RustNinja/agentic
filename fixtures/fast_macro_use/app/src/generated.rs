pub struct GeneratedMessage {
    value: u32,
}

impl GeneratedMessage {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

#[allow(dead_code)]
pub fn dead_generated() -> u32 {
    999
}
