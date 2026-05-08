#![allow(dead_code)]

mod dead;
mod live;

pub mod prelude {
    pub use crate::dead::{dead_report, DeadGateway};
    pub use crate::live::{selected_report, GatewayRecord};
}

pub use dead::{dead_report, DeadGateway};
pub use live::{selected_report, GatewayRecord};
