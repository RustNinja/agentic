use e::mix as renamed_mix;

pub type Score = e::Score;

pub const SALT: Score = e::DEFAULT_SEED;

pub struct Worker {
    factor: Score,
}

impl Worker {
    pub const FACTOR: Score = 2;

    pub fn new() -> Self {
        Self {
            factor: e::seed() + Self::FACTOR,
        }
    }

    pub fn run(&self, value: Score) -> Score {
        e::finish(value + self.factor)
    }

    pub fn unused_method(&self) -> Score {
        e::unused_leaf(self.factor)
    }
}

pub fn hash(value: Score) -> Score {
    renamed_mix(value) + SALT
}

pub fn normalize(value: Score) -> Score {
    hash(value).abs()
}

pub fn shared(value: Score) -> Score {
    hash(value) - 2
}

pub fn unused_public(value: Score) -> Score {
    unused_private(value)
}

fn unused_private(value: Score) -> Score {
    e::unused_leaf(value)
}
