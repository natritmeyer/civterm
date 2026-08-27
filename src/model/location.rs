#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub x: u16,
    pub y: u16,
}

impl Location {
    pub fn new(x: u16, y: u16) -> Self {
        Location { x, y }
    }
}
