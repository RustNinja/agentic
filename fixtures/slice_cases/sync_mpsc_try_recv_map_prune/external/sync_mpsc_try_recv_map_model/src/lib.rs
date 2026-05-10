mod dead;
mod live;

pub use dead::dead_sync_mpsc_try_recv_map;
pub use live::selected_sync_mpsc_try_recv_map;
