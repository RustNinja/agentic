use d::hash as renamed_hash;
use d::Transform;

pub fn compute(value: i32) -> i32 {
    let worker = d::Worker::new();
    helper(value, d::Mode::Fast)
        + worker.run(value)
        + worker.transform(value)
        + renamed_hash(value, d::Mode::Fast)
        + d::shared(value)
}

fn helper(value: i32, mode: d::Mode) -> i32 {
    d::hash(value, mode)
}

pub fn unused_public() -> i32 {
    d::unused_public(1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_test_that_must_not_be_exported() {
        assert_eq!(super::compute(1), super::compute(1));
    }
}
