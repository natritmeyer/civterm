use crate::model::geography::Geography;
use crate::model::geography_improvement::GeographyImprovement;

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub geography: Geography,
    pub discovered: bool,
    irrigated: bool,
    mined: bool,
    has_road: bool,
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

    pub fn irrigate(&mut self) -> Result<(), GeographyImprovement> {
        self.apply_improvement(GeographyImprovement::Irrigation)
    }

    pub fn mine(&mut self) -> Result<(), GeographyImprovement> {
        self.apply_improvement(GeographyImprovement::Mine)
    }

    pub fn build_road(&mut self) -> Result<(), GeographyImprovement> {
        self.apply_improvement(GeographyImprovement::Road)
    }

    fn apply_improvement(
        &mut self,
        improvement: GeographyImprovement,
    ) -> Result<(), GeographyImprovement> {
        if !self.geography.supports(improvement) {
            return Err(improvement);
        }
        match improvement {
            GeographyImprovement::Irrigation => self.irrigated = true,
            GeographyImprovement::Mine => self.mined = true,
            GeographyImprovement::Road => self.has_road = true,
        }
        Ok(())
    }

    pub fn yields_food(&self) -> u8 {
        let mut food = self.geography.yields_food();
        if self.irrigated {
            food += 1;
        }
        food
    }

    pub fn yields_resources(&self) -> u8 {
        let mut resources = self.geography.yields_resources();
        if self.mined {
            resources += 1;
        }
        resources
    }

    pub fn yields_trade(&self) -> u8 {
        let mut trade = self.geography.yields_trade();
        if self.has_road {
            trade += 1;
        }
        trade
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

    #[test]
    fn default_yields_match_geography() {
        let ocean = Tile::new(Geography::Ocean);
        assert_eq!(ocean.yields_food(), 2);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);
    }

    #[test]
    fn irrigation_adds_food_on_supporting_geography() {
        let mut tile = Tile::new(Geography::Plains);
        tile.irrigate().unwrap();
        assert_eq!(tile.yields_food(), 3);
        assert_eq!(tile.yields_resources(), 1);
        assert_eq!(tile.yields_trade(), 0);
    }

    #[test]
    fn mine_adds_resources_on_supporting_geography() {
        let mut tile = Tile::new(Geography::Mountain);
        tile.mine().unwrap();
        assert_eq!(tile.yields_food(), 0);
        assert_eq!(tile.yields_resources(), 2);
        assert_eq!(tile.yields_trade(), 1);
    }

    #[test]
    fn road_adds_trade_on_land() {
        let mut tile = Tile::new(Geography::Grassland);
        tile.build_road().unwrap();
        assert_eq!(tile.yields_food(), 2);
        assert_eq!(tile.yields_resources(), 0);
        assert_eq!(tile.yields_trade(), 2);
    }

    #[test]
    fn unsupported_improvements_are_rejected() {
        let mut forest = Tile::new(Geography::Forest);
        assert_eq!(forest.irrigate(), Err(GeographyImprovement::Irrigation));
        assert_eq!(forest.yields_food(), 1);

        let mut ocean = Tile::new(Geography::Ocean);
        assert_eq!(ocean.mine(), Err(GeographyImprovement::Mine));
        assert_eq!(ocean.build_road(), Err(GeographyImprovement::Road));
        assert_eq!(ocean.yields_food(), 2);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);
    }

    #[test]
    fn combined_improvements() {
        let mut tile = Tile::new(Geography::Desert);
        tile.irrigate().unwrap();
        tile.mine().unwrap();
        tile.build_road().unwrap();
        assert_eq!(tile.yields_food(), 1);
        assert_eq!(tile.yields_resources(), 2);
        assert_eq!(tile.yields_trade(), 2);
    }
}
