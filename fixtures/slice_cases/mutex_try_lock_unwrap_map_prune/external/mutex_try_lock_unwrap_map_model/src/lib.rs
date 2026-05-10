mod dead;
mod live;

pub use dead::dead_mutex_try_lock_unwrap_map;
pub use live::selected_mutex_try_lock_unwrap_map;
