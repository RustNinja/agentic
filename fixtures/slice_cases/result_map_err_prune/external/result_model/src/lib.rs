mod dead;
mod live;

pub use dead::{dead_result, DeadResult};
pub use live::{parse_result, ResultError, ResultValue};
