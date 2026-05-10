use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    fs::write(
        out_dir.join("slice_case_generated.rs"),
        "pub fn generated_value() -> u32 { super::out_dir_generated_helper() }\n",
    )
    .expect("generated source should be writable");
}

