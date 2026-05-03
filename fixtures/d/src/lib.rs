use e::mix as renamed_mix;

pub type Score = e::Score;

pub const SALT: Score = e::DEFAULT_SEED;

pub enum Mode {
    Fast,
    Slow,
}

pub enum UnusedEnum {
    Dead,
}

pub struct Worker {
    factor: Score,
}

pub trait Transform {
    fn transform(&self, value: Score) -> Score;
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

impl Default for Worker {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for Worker {
    fn transform(&self, value: Score) -> Score {
        self.run(value) + renamed_mix(value)
    }
}

pub fn hash(value: Score, mode: Mode) -> Score {
    match mode {
        Mode::Fast => renamed_mix(value) + SALT,
        Mode::Slow => renamed_mix(value) - SALT,
    }
}

pub fn normalize(value: Score) -> Score {
    hash(value, Mode::Fast).abs()
}

pub fn shared(value: Score) -> Score {
    hash(value, Mode::Fast) - 2
}

pub fn unused_public(value: Score) -> Score {
    unused_private(value)
}

fn unused_private(value: Score) -> Score {
    e::unused_leaf(value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_test_that_must_not_be_exported() {
        assert_eq!(2 + 2, 4);
    }
}
