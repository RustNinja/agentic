pub type Score = i32;

pub const DEFAULT_SEED: Score = 7;

pub fn mix(value: Score) -> Score {
    value * 3 + DEFAULT_SEED
}

pub fn finish(value: Score) -> Score {
    mix(value) - 1
}

pub fn seed() -> Score {
    DEFAULT_SEED
}

pub fn unused_leaf(value: Score) -> Score {
    value + 1000
}

pub fn tempting_but_unused() -> Score {
    unused_leaf(1)
}
