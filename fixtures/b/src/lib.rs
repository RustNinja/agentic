use d::hash as renamed_hash;

pub fn compute(value: i32) -> i32 {
    let worker = d::Worker::new();
    helper(value) + worker.run(value) + renamed_hash(value) + d::shared(value)
}

fn helper(value: i32) -> i32 {
    d::hash(value)
}

pub fn unused_public() -> i32 {
    d::unused_public(1)
}
