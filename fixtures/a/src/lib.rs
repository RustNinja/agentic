use b as renamed_b;
use opensourced::opensourced;

#[opensourced]
pub fn open_source_entry(value: i32) -> i32 {
    renamed_b::compute(value) + c::adjust(value)
}

pub fn internal_entry(value: i32) -> i32 {
    b::unused_public() + value
}
