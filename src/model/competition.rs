#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Competition {
    rivals: u8,
}

impl Competition {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 7;

    pub fn new(rivals: u8) -> Self {
        Competition {
            rivals: rivals.clamp(Self::MIN, Self::MAX),
        }
    }

    pub fn rivals(&self) -> u8 {
        self.rivals
    }

    /// Total civilizations on the map, including the player's own.
    pub fn total_civilizations(&self) -> u8 {
        self.rivals + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_from_one_to_seven_is_valid() {
        for rivals in Competition::MIN..=Competition::MAX {
            assert_eq!(Competition::new(rivals).rivals(), rivals);
        }
    }

    #[test]
    fn rivals_are_clamped_to_the_supported_range() {
        assert_eq!(Competition::new(0).rivals(), Competition::MIN);
        assert_eq!(Competition::new(99).rivals(), Competition::MAX);
    }

    #[test]
    fn total_civilizations_includes_the_player() {
        assert_eq!(Competition::new(1).total_civilizations(), 2);
        assert_eq!(Competition::new(7).total_civilizations(), 8);
    }
}
