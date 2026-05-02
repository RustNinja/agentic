use opensourced::opensourced;

#[opensourced]
pub struct Exposed {
    value: usize,
}

#[opensourced]
pub enum Choice {
    One,
}

#[opensourced]
pub trait Worker {
    fn work(&self) -> Choice;
}

#[opensourced]
pub mod grouped {
    pub const VALUE: usize = 7;
}

impl Exposed {
    #[opensourced]
    pub fn new(value: usize) -> Self {
        Self { value }
    }
}

impl Worker for Exposed {
    fn work(&self) -> Choice {
        let _ = self.value;
        Choice::One
    }
}

#[test]
fn marker_attribute_is_transparent_on_non_function_items() {
    let exposed = Exposed::new(1);
    assert!(matches!(exposed.work(), Choice::One));
    assert_eq!(grouped::VALUE, 7);
}
