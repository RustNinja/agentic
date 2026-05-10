mod dead;
mod live;

pub use dead::dead_mpsc_recv_timeout_map;
pub use live::selected_mpsc_recv_timeout_map;
