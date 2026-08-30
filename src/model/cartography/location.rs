#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub x: u16,
    pub y: u16,
}

impl Location {
    pub fn new(x: u16, y: u16) -> Self {
        Location { x, y }
    }

    pub fn is_adjacent(&self, other: &Location) -> bool {
        self != other && self.x.abs_diff(other.x) <= 1 && self.y.abs_diff(other.y) <= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_is_not_adjacent_to_itself() {
        let location = Location::new(2, 2);
        assert!(!location.is_adjacent(&location));
    }

    #[test]
    fn orthogonal_neighbours_are_adjacent() {
        let origin = Location::new(2, 2);
        assert!(origin.is_adjacent(&Location::new(3, 2)));
        assert!(origin.is_adjacent(&Location::new(2, 3)));
    }

    #[test]
    fn diagonal_neighbours_are_adjacent() {
        let origin = Location::new(2, 2);
        assert!(origin.is_adjacent(&Location::new(3, 3)));
        assert!(origin.is_adjacent(&Location::new(1, 1)));
    }

    #[test]
    fn distant_locations_are_not_adjacent() {
        let origin = Location::new(2, 2);
        assert!(!origin.is_adjacent(&Location::new(4, 2)));
        assert!(!origin.is_adjacent(&Location::new(2, 4)));
        assert!(!origin.is_adjacent(&Location::new(5, 5)));
    }
}
