use crate::model::geography::{SpecialResource, Terrain, TerrainImprovement};

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub terrain: Terrain,
    irrigated: bool,
    mined: bool,
    has_road: bool,
    resource: Option<SpecialResource>,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Tile {
            terrain,
            irrigated: false,
            mined: false,
            has_road: false,
            resource: None,
        }
    }

    pub fn place_resource(&mut self, resource: SpecialResource) -> Result<(), SpecialResource> {
        if self.terrain.supports_resource(resource) {
            self.resource = Some(resource);
            Ok(())
        } else {
            Err(resource)
        }
    }

    pub fn irrigate(&mut self) -> Result<(), TerrainImprovement> {
        self.apply_improvement(TerrainImprovement::Irrigation)
    }

    pub fn mine(&mut self) -> Result<(), TerrainImprovement> {
        self.apply_improvement(TerrainImprovement::Mine)
    }

    pub fn build_road(&mut self) -> Result<(), TerrainImprovement> {
        self.apply_improvement(TerrainImprovement::Road)
    }

    pub fn apply_improvement(
        &mut self,
        improvement: TerrainImprovement,
    ) -> Result<(), TerrainImprovement> {
        if !self.terrain.supports(improvement) {
            return Err(improvement);
        }
        match improvement {
            TerrainImprovement::Irrigation => self.irrigated = true,
            TerrainImprovement::Mine => self.mined = true,
            TerrainImprovement::Road => self.has_road = true,
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
        let mut food = self.terrain.yields_food();
        if self.irrigated {
            food += 1;
        }
        if let Some(resource) = self.resource {
            food += resource.yields_food();
        }
        food
    }

    pub fn yields_resources(&self) -> u8 {
        let mut resources = self.terrain.yields_resources();
        if self.mined {
            resources += 1;
        }
        if let Some(resource) = self.resource {
            resources += resource.yields_resources();
        }
        resources
    }

    pub fn yields_trade(&self) -> u8 {
        let mut trade = self.terrain.yields_trade();
        if self.has_road {
            trade += 1;
        }
        if let Some(resource) = self.resource {
            trade += resource.yields_trade();
        }
        trade
    }

    /// The special resource on this tile, if any.
    pub fn resource(&self) -> Option<SpecialResource> {
        self.resource
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_starts_without_improvements() {
        let tile = Tile::new(Terrain::Grassland);
        assert!(!tile.irrigated);
        assert!(!tile.mined);
        assert!(!tile.has_road);
        assert_eq!(tile.resource, None);
        assert!(!tile.is_irrigated());
        assert!(!tile.is_mined());
        assert!(!tile.has_road());
    }

    #[test]
    fn default_yields_match_geography() {
        let ocean = Tile::new(Terrain::Ocean);
        assert_eq!(ocean.yields_food(), 2);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);
    }

    #[test]
    fn irrigation_adds_food_on_supporting_geography() {
        let mut tile = Tile::new(Terrain::Plains);
        tile.irrigate().unwrap();
        assert_eq!(tile.yields_food(), 3);
        assert_eq!(tile.yields_resources(), 1);
        assert_eq!(tile.yields_trade(), 0);
    }

    #[test]
    fn mine_adds_resources_on_supporting_geography() {
        let mut tile = Tile::new(Terrain::Mountain);
        tile.mine().unwrap();
        assert_eq!(tile.yields_food(), 0);
        assert_eq!(tile.yields_resources(), 2);
        assert_eq!(tile.yields_trade(), 1);
    }

    #[test]
    fn road_adds_trade_on_land() {
        let mut tile = Tile::new(Terrain::Grassland);
        tile.build_road().unwrap();
        assert_eq!(tile.yields_food(), 2);
        assert_eq!(tile.yields_resources(), 0);
        assert_eq!(tile.yields_trade(), 2);
    }

    #[test]
    fn unsupported_improvements_are_rejected() {
        let mut forest = Tile::new(Terrain::Forest);
        assert_eq!(forest.irrigate(), Err(TerrainImprovement::Irrigation));
        assert_eq!(forest.yields_food(), 1);

        let mut ocean = Tile::new(Terrain::Ocean);
        assert_eq!(ocean.mine(), Err(TerrainImprovement::Mine));
        assert_eq!(ocean.build_road(), Err(TerrainImprovement::Road));
        assert_eq!(ocean.yields_food(), 2);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);
    }

    #[test]
    fn combined_improvements() {
        let mut tile = Tile::new(Terrain::Desert);
        tile.irrigate().unwrap();
        tile.mine().unwrap();
        tile.build_road().unwrap();
        assert_eq!(tile.yields_food(), 1);
        assert_eq!(tile.yields_resources(), 2);
        assert_eq!(tile.yields_trade(), 2);
    }

    #[test]
    fn resource_placement_is_gated_by_geography() {
        let mut ocean = Tile::new(Terrain::Ocean);
        ocean.place_resource(SpecialResource::Fish).unwrap();
        assert_eq!(ocean.yields_food(), 3);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);

        let mut hills = Tile::new(Terrain::Hills);
        hills.place_resource(SpecialResource::Coal).unwrap();
        assert_eq!(hills.yields_food(), 1);
        assert_eq!(hills.yields_resources(), 2);
        assert_eq!(hills.yields_trade(), 0);

        let mut jungle = Tile::new(Terrain::Jungle);
        jungle.place_resource(SpecialResource::Gems).unwrap();
        assert_eq!(jungle.yields_food(), 1);
        assert_eq!(jungle.yields_resources(), 1);
        assert_eq!(jungle.yields_trade(), 2);
    }

    #[test]
    fn resource_rejected_on_unsupporting_geography() {
        let mut ocean = Tile::new(Terrain::Ocean);
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
        let mut tile = Tile::new(Terrain::Desert);
        tile.place_resource(SpecialResource::Oasis).unwrap();
        tile.irrigate().unwrap();
        tile.mine().unwrap();
        tile.build_road().unwrap();
        assert!(tile.is_irrigated());
        assert!(tile.is_mined());
        assert!(tile.has_road());
        assert_eq!(tile.yields_food(), 4);
        assert_eq!(tile.yields_resources(), 2);
        assert_eq!(tile.yields_trade(), 2);
    }

    #[test]
    fn apply_improvement_sets_the_gettable_state() {
        let mut desert = Tile::new(Terrain::Desert);
        desert
            .apply_improvement(TerrainImprovement::Irrigation)
            .unwrap();
        desert.apply_improvement(TerrainImprovement::Mine).unwrap();
        desert.apply_improvement(TerrainImprovement::Road).unwrap();
        assert!(desert.is_irrigated());
        assert!(desert.is_mined());
        assert!(desert.has_road());
        assert_eq!(desert.yields_food(), 1);
        assert_eq!(desert.yields_resources(), 2);
        assert_eq!(desert.yields_trade(), 2);

        let mut grassland = Tile::new(Terrain::Grassland);
        grassland
            .apply_improvement(TerrainImprovement::Road)
            .unwrap();
        assert!(grassland.has_road());
        assert!(!grassland.is_irrigated());
        assert!(!grassland.is_mined());
        assert_eq!(grassland.yields_trade(), 2);
    }

    #[test]
    fn improvements_are_idempotent() {
        let mut desert = Tile::new(Terrain::Desert);
        desert.irrigate().unwrap();
        desert.irrigate().unwrap();
        assert_eq!(desert.yields_food(), 1);
        assert_eq!(desert.yields_resources(), 1);
        assert_eq!(desert.yields_trade(), 1);

        desert.mine().unwrap();
        desert.mine().unwrap();
        assert_eq!(desert.yields_resources(), 2);

        desert.build_road().unwrap();
        desert.build_road().unwrap();
        assert_eq!(desert.yields_trade(), 2);
    }

    #[test]
    fn rejected_improvement_leaves_state_unchanged() {
        let mut forest = Tile::new(Terrain::Forest);
        assert_eq!(forest.mine(), Err(TerrainImprovement::Mine));
        assert_eq!(forest.irrigate(), Err(TerrainImprovement::Irrigation));
        assert!(!forest.is_mined());
        assert!(!forest.is_irrigated());

        let mut ocean = Tile::new(Terrain::Ocean);
        assert_eq!(ocean.build_road(), Err(TerrainImprovement::Road));
        assert!(!ocean.has_road());
        assert_eq!(ocean.yields_food(), 2);
        assert_eq!(ocean.yields_resources(), 0);
        assert_eq!(ocean.yields_trade(), 2);
    }

    #[test]
    fn each_special_resource_adds_its_yields_on_supporting_geography() {
        let mut forest = Tile::new(Terrain::Forest);
        forest.place_resource(SpecialResource::Game).unwrap();
        assert_eq!(forest.yields_food(), 2);
        assert_eq!(forest.yields_resources(), 2);
        assert_eq!(forest.yields_trade(), 0);

        let mut tundra = Tile::new(Terrain::Tundra);
        tundra.place_resource(SpecialResource::Game).unwrap();
        assert_eq!(tundra.yields_food(), 2);
        assert_eq!(tundra.yields_resources(), 0);
        assert_eq!(tundra.yields_trade(), 1);

        let mut plains = Tile::new(Terrain::Plains);
        plains.place_resource(SpecialResource::Horses).unwrap();
        assert_eq!(plains.yields_food(), 2);
        assert_eq!(plains.yields_resources(), 2);
        assert_eq!(plains.yields_trade(), 0);

        let mut mountain = Tile::new(Terrain::Mountain);
        mountain.place_resource(SpecialResource::Gold).unwrap();
        assert_eq!(mountain.yields_food(), 0);
        assert_eq!(mountain.yields_resources(), 1);
        assert_eq!(mountain.yields_trade(), 3);

        let mut desert = Tile::new(Terrain::Desert);
        desert.place_resource(SpecialResource::Oasis).unwrap();
        assert_eq!(desert.yields_food(), 3);
        assert_eq!(desert.yields_resources(), 1);
        assert_eq!(desert.yields_trade(), 1);

        let mut swamp = Tile::new(Terrain::Swamp);
        swamp.place_resource(SpecialResource::Oil).unwrap();
        assert_eq!(swamp.yields_food(), 1);
        assert_eq!(swamp.yields_resources(), 2);
        assert_eq!(swamp.yields_trade(), 1);
    }

    #[test]
    fn a_resource_can_be_placed_over_an_existing_one() {
        let mut tundra = Tile::new(Terrain::Tundra);
        tundra.place_resource(SpecialResource::Game).unwrap();
        assert_eq!(tundra.yields_food(), 2);
        tundra.place_resource(SpecialResource::Game).unwrap();
        assert_eq!(tundra.yields_food(), 2);
    }

    #[test]
    fn road_widens_trade_on_every_land_terrain() {
        for terrain in [
            Terrain::Grassland,
            Terrain::Plains,
            Terrain::Forest,
            Terrain::Hills,
            Terrain::Mountain,
            Terrain::Desert,
            Terrain::Tundra,
            Terrain::Swamp,
            Terrain::Jungle,
        ] {
            let mut tile = Tile::new(terrain);
            tile.build_road().unwrap();
            assert_eq!(
                tile.yields_trade(),
                terrain.yields_trade() + 1,
                "road should add trade on {terrain:?}"
            );
        }
    }
}
