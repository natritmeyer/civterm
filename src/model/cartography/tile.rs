use crate::model::geography::{Geography, GeographyImprovement, SpecialResource};

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub geography: Geography,
    irrigated: bool,
    mined: bool,
    has_road: bool,
    resource: Option<SpecialResource>,
}

impl Tile {
    pub fn new(geography: Geography) -> Self {
        Tile {
            geography,
            irrigated: false,
            mined: false,
            has_road: false,
            resource: None,
        }
    }

    pub fn place_resource(&mut self, resource: SpecialResource) -> Result<(), SpecialResource> {
        if self.geography.supports_resource(resource) {
            self.resource = Some(resource);
            Ok(())
        } else {
            Err(resource)
        }
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

    pub fn apply_improvement(
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

    pub fn is_irrigated(&self) -> bool {
        self.irrigated
    }

    pub fn is_mined(&self) -> bool {
        self.mined
    }

    pub fn has_road(&self) -> bool {
        self.has_road
    }

    pub fn yields_food(&self) -> u8 {
        let mut food = self.geography.yields_food();
        if self.irrigated {
            food += 1;
        }
        if let Some(resource) = self.resource {
            food += resource.yields_food();
        }
        food
    }

    pub fn yields_resources(&self) -> u8 {
        let mut resources = self.geography.yields_resources();
        if self.mined {
            resources += 1;
        }
        if let Some(resource) = self.resource {
            resources += resource.yields_resources();
        }
        resources
    }

    pub fn yields_trade(&self) -> u8 {
        let mut trade = self.geography.yields_trade();
        if self.has_road {
            trade += 1;
        }
        if let Some(resource) = self.resource {
            trade += resource.yields_trade();
        }
        trade
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_starts_without_improvements() {
        let tile = Tile::new(Geography::Grassland);
        assert!(!tile.irrigated);
        assert!(!tile.mined);
        assert!(!tile.has_road);
        assert_eq!(tile.resource, None);
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

    #[test]
    fn resource_placement_is_gated_by_geography() {
        let mut ocean = Tile::new(Geography::Ocean);
        ocean.place_resource(SpecialResource::Fish).unwrap();
        assert_eq!(ocean.yields_food(), 3);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);

        let mut hills = Tile::new(Geography::Hills);
        hills.place_resource(SpecialResource::Coal).unwrap();
        assert_eq!(hills.yields_food(), 1);
        assert_eq!(hills.yields_resources(), 2);
        assert_eq!(hills.yields_trade(), 0);

        let mut jungle = Tile::new(Geography::Jungle);
        jungle.place_resource(SpecialResource::Gems).unwrap();
        assert_eq!(jungle.yields_food(), 1);
        assert_eq!(jungle.yields_resources(), 1);
        assert_eq!(jungle.yields_trade(), 2);
    }

    #[test]
    fn resource_rejected_on_unsupporting_geography() {
        let mut ocean = Tile::new(Geography::Ocean);
        assert_eq!(
            ocean.place_resource(SpecialResource::Coal),
            Err(SpecialResource::Coal)
        );
        assert_eq!(ocean.yields_food(), 2);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);
    }

    #[test]
    fn resource_combines_with_improvements() {
        let mut tile = Tile::new(Geography::Desert);
        tile.place_resource(SpecialResource::Oasis).unwrap();
        tile.irrigate().unwrap();
        tile.mine().unwrap();
        tile.build_road().unwrap();
        assert_eq!(tile.yields_food(), 4);
        assert_eq!(tile.yields_resources(), 2);
        assert_eq!(tile.yields_trade(), 2);
    }
}
