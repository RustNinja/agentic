mod dead;
mod live;

pub use dead::{dead_poll, DeadPoller};
pub use live::{selected_poll, PollFrame, Poller};
