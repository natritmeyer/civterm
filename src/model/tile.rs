use crate::model::geography::Geography;

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub geography: Geography,
    pub discovered: bool,
    pub irrigated: bool,
    pub mined: bool,
    pub has_road: bool,
}

impl Tile {
    pub fn new(geography: Geography) -> Self {
        Tile {
            geography,
            discovered: false,
            irrigated: false,
            mined: false,
            has_road: false,
        }
    }

    pub fn discover(&mut self) {
        self.discovered = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_starts_undiscovered() {
        let tile = Tile::new(Geography::Grassland);
        assert!(!tile.discovered);
    }

    #[test]
    fn tile_starts_without_improvements() {
        let tile = Tile::new(Geography::Grassland);
        assert!(!tile.irrigated);
        assert!(!tile.mined);
        assert!(!tile.has_road);
    }

    #[test]
    fn tile_can_be_discovered() {
        let mut tile = Tile::new(Geography::Ocean);
        tile.discover();
        assert!(tile.discovered);
    }
}
