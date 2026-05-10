mod dead;
mod live;

pub use dead::dead_rwlock_try_read_map;
pub use live::selected_rwlock_try_read_map;
