mod dead;
mod live;

pub use dead::{dead_registry_metric, DeadRegistryEntry};
pub use live::{list_entries, lookup_entry, register_entry, remove_entry, RegistryEntry};
