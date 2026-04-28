pub fn adjust(value: i32) -> i32 {
    let worker = d::Worker::new();
    worker.run(d::normalize(value))
}

pub fn unused() -> i32 {
    d::unused_public(2)
}
