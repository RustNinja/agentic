mod generated {
    include!(concat!(env!("OUT_DIR"), "/slice_case_generated.rs"));
}

pub fn selected_generated_value() -> u32 {
    generated::generated_value()
}

pub fn out_dir_generated_helper() -> u32 {
    41
}

pub fn dead_generated_value() -> u32 {
    dead_out_dir_helper()
}

pub fn dead_out_dir_helper() -> u32 {
    99
}

