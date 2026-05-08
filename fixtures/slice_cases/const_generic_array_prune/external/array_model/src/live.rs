pub const LIVE_LEN: usize = 4;

pub struct ArrayPayload<const N: usize> {
    values: [u8; N],
}

impl<const N: usize> ArrayPayload<N> {
    pub fn render(self) -> String {
        format!("array:{}:{}", N, self.values.iter().sum::<u8>())
    }

    pub fn dead_method(self) -> String {
        format!("dead-array:{N}")
    }
}

pub fn selected_array(raw: &str) -> ArrayPayload<LIVE_LEN> {
    ArrayPayload {
        values: [raw.len() as u8; LIVE_LEN],
    }
}

pub fn dead_live_array(raw: &str) -> String {
    selected_array(raw).dead_method()
}
