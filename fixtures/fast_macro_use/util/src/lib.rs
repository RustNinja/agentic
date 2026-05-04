pub trait Useful {
    fn useful(&self) -> u32;
}

pub struct UtilValue(pub u32);

impl Useful for UtilValue {
    fn useful(&self) -> u32 {
        self.0 + helper()
    }
}

pub fn make_util(value: u32) -> UtilValue {
    UtilValue(value)
}

fn helper() -> u32 {
    2
}

pub fn dead_util() -> u32 {
    99
}
