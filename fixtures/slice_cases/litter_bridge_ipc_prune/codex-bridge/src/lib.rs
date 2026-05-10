#![allow(dead_code)]

pub mod ipc;
pub mod ssh;

pub fn dead_bridge_entry() -> String {
    ssh::dead_ssh_entry().to_string()
}

