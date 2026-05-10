use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    fs::write(
        out_dir.join("litter_codegen_bindings.rs"),
        "pub fn generated_event(label: &str) -> super::GeneratedEvent { super::generated_event_helper(label) }\n",
    )
    .expect("generated source should be writable");
}

