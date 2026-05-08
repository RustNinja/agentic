pub const DEAD_LEN: usize = 9;

pub struct DeadArray {
    values: [u8; DEAD_LEN],
}

pub fn dead_array(raw: &str) -> String {
    let array = DeadArray {
        values: [raw.len() as u8; DEAD_LEN],
    };
    format!("dead-array-model:{}", array.values.len())
}
