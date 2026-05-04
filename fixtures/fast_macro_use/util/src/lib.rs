pub trait Useful {
    type Output;

    const BONUS: u32;

    fn useful(&self) -> Self::Output;
}

pub struct UtilValue(pub u32);

impl Useful for UtilValue {
    type Output = u32;

    const BONUS: u32 = 6;

    fn useful(&self) -> Self::Output {
        self.0 + helper() + Self::BONUS
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
