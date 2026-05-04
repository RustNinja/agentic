use std::{fmt, ops::Deref, str::FromStr};

pub trait Useful {
    type Output;

    const BONUS: u32;

    fn useful(&self) -> Self::Output;
}

pub trait Transform<T> {
    type Output;

    fn transform(&self, input: T) -> Self::Output;
}

pub trait DescribeValue {
    type Value;

    const LABEL: &'static str;

    fn describe_value(&self) -> Self::Value;
}

pub struct UtilValue(pub u32);

impl Useful for UtilValue {
    type Output = u32;

    const BONUS: u32 = 6;

    fn useful(&self) -> Self::Output {
        self.0 + helper() + Self::BONUS
    }
}

impl<T> Transform<T> for UtilValue
where
    T: Copy + Into<u32>,
{
    type Output = u32;

    fn transform(&self, input: T) -> Self::Output {
        self.0 + input.into() + helper()
    }
}

impl DescribeValue for UtilValue {
    type Value = String;

    const LABEL: &'static str = "util";

    fn describe_value(&self) -> Self::Value {
        format!("{}:{}", Self::LABEL, self.0)
    }
}

impl FromStr for UtilValue {
    type Err = std::num::ParseIntError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input.parse::<u32>().map(Self)
    }
}

impl fmt::Display for UtilValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "util-{}", self.0)
    }
}

impl Deref for UtilValue {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
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
