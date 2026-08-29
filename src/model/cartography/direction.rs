#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    pub fn delta(&self) -> (isize, isize) {
        match self {
            Direction::N => (0, -1),
            Direction::NE => (1, -1),
            Direction::E => (1, 0),
            Direction::SE => (1, 1),
            Direction::S => (0, 1),
            Direction::SW => (-1, 1),
            Direction::W => (-1, 0),
            Direction::NW => (-1, -1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_deltas() {
        assert_eq!(Direction::N.delta(), (0, -1));
        assert_eq!(Direction::NE.delta(), (1, -1));
        assert_eq!(Direction::E.delta(), (1, 0));
        assert_eq!(Direction::SE.delta(), (1, 1));
        assert_eq!(Direction::S.delta(), (0, 1));
        assert_eq!(Direction::SW.delta(), (-1, 1));
        assert_eq!(Direction::W.delta(), (-1, 0));
        assert_eq!(Direction::NW.delta(), (-1, -1));
    }
}
