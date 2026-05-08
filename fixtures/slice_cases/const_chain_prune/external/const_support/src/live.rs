pub const BASE: usize = 2;
pub const SCALE: usize = BASE + 3;
pub static LABEL: &str = "live-const";

pub struct ConstRecord {
    value: usize,
}

impl ConstRecord {
    pub const OFFSET: usize = SCALE * 2;

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len() + Self::OFFSET,
        }
    }

    pub fn render(&self) -> String {
        format!("{LABEL}:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-const:{}", self.value)
    }
}

pub fn selected_const(raw: &str) -> String {
    ConstRecord::new(raw).render()
}

pub fn dead_live_const(raw: &str) -> String {
    ConstRecord::new(raw).dead_method()
}
