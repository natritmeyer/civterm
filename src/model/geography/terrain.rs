use super::movement_category::MovementCategory;
use super::special_resource::SpecialResource;
use super::terrain_improvement::TerrainImprovement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Ocean,
    Grassland,
    Plains,
    Forest,
    Hills,
    Mountain,
    Desert,
    Tundra,
    Swamp,
    Jungle,
}

impl Terrain {
    pub fn movement_class(&self) -> MovementCategory {
        match self {
            Terrain::Ocean
            | Terrain::Grassland
            | Terrain::Plains
            | Terrain::Desert
            | Terrain::Tundra => MovementCategory::Open,
            Terrain::Forest | Terrain::Hills | Terrain::Swamp | Terrain::Jungle => {
                MovementCategory::Dense
            }
            Terrain::Mountain => MovementCategory::Mountain,
        }
    }

    pub fn movement_cost(&self) -> u8 {
        self.movement_class().movement_cost()
    }

    pub fn is_water(&self) -> bool {
        matches!(self, Terrain::Ocean)
    }

    pub fn is_land(&self) -> bool {
        !self.is_water()
    }

    pub fn supports(&self, improvement: TerrainImprovement) -> bool {
        match improvement {
            TerrainImprovement::Irrigation => matches!(
                self,
                Terrain::Grassland
                    | Terrain::Plains
                    | Terrain::Desert
                    | Terrain::Swamp
                    | Terrain::Jungle
            ),
            TerrainImprovement::Mine => {
                matches!(self, Terrain::Hills | Terrain::Mountain | Terrain::Desert)
            }
            TerrainImprovement::Road => self.is_land(),
        }
    }

    pub fn supports_resource(&self, resource: SpecialResource) -> bool {
        match resource {
            SpecialResource::Coal => matches!(self, Terrain::Hills),
            SpecialResource::Fish => matches!(self, Terrain::Ocean),
            SpecialResource::Game => matches!(self, Terrain::Forest | Terrain::Tundra),
            SpecialResource::Gems => matches!(self, Terrain::Jungle),
            SpecialResource::Gold => matches!(self, Terrain::Mountain),
            SpecialResource::Horses => matches!(self, Terrain::Plains),
            SpecialResource::Oasis => matches!(self, Terrain::Desert),
            SpecialResource::Oil => matches!(self, Terrain::Swamp),
        }
    }

    pub fn yields_food(&self) -> u8 {
        match self {
            Terrain::Ocean | Terrain::Grassland | Terrain::Plains => 2,
            Terrain::Forest
            | Terrain::Hills
            | Terrain::Tundra
            | Terrain::Swamp
            | Terrain::Jungle => 1,
            Terrain::Mountain | Terrain::Desert => 0,
        }
    }

    pub fn yields_resources(&self) -> u8 {
        match self {
            Terrain::Forest => 2,
            Terrain::Plains
            | Terrain::Hills
            | Terrain::Mountain
            | Terrain::Desert
            | Terrain::Jungle => 1,
            Terrain::Ocean | Terrain::Grassland | Terrain::Tundra | Terrain::Swamp => 0,
        }
    }

    pub fn yields_trade(&self) -> u8 {
        match self {
            Terrain::Ocean => 2,
            Terrain::Grassland
            | Terrain::Mountain
            | Terrain::Desert
            | Terrain::Tundra
            | Terrain::Swamp => 1,
            Terrain::Plains | Terrain::Forest | Terrain::Hills | Terrain::Jungle => 0,
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Terrain::Ocean => '~',
            Terrain::Grassland => 'v',
            Terrain::Plains => '.',
            Terrain::Forest => '#',
            Terrain::Hills => '^',
            Terrain::Mountain => 'M',
            Terrain::Desert => 'd',
            Terrain::Tundra => 't',
            Terrain::Swamp => 's',
            Terrain::Jungle => 'T',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATER: [Terrain; 1] = [Terrain::Ocean];

    const OPEN: [Terrain; 4] = [
        Terrain::Grassland,
        Terrain::Plains,
        Terrain::Desert,
        Terrain::Tundra,
    ];

    const DENSE: [Terrain; 4] = [
        Terrain::Forest,
        Terrain::Hills,
        Terrain::Swamp,
        Terrain::Jungle,
    ];

    #[test]
    fn open_terrains_are_open_class() {
        for g in OPEN {
            assert_eq!(g.movement_class(), MovementCategory::Open);
        }
    }

    #[test]
    fn dense_terrains_are_dense_class() {
        for g in DENSE {
            assert_eq!(g.movement_class(), MovementCategory::Dense);
        }
    }

    #[test]
    fn mountain_is_mountain_class() {
        assert_eq!(
            Terrain::Mountain.movement_class(),
            MovementCategory::Mountain
        );
    }

    #[test]
    fn ocean_is_open_class() {
        for g in WATER {
            assert_eq!(g.movement_class(), MovementCategory::Open);
        }
    }

    #[test]
    fn open_terrain_costs_one() {
        for g in OPEN {
            assert_eq!(g.movement_cost(), 1);
        }
    }

    #[test]
    fn dense_terrain_costs_two() {
        for g in DENSE {
            assert_eq!(g.movement_cost(), 2);
        }
    }

    #[test]
    fn mountains_cost_three() {
        assert_eq!(Terrain::Mountain.movement_cost(), 3);
    }

    #[test]
    fn only_ocean_is_water() {
        for g in WATER {
            assert!(g.is_water());
            assert!(!g.is_land());
        }
        for g in OPEN.iter().chain(DENSE.iter()).chain([&Terrain::Mountain]) {
            assert!(!g.is_water());
            assert!(g.is_land());
        }
    }

    #[test]
    fn irrigation_supported_terrains() {
        for g in [
            Terrain::Grassland,
            Terrain::Plains,
            Terrain::Desert,
            Terrain::Swamp,
            Terrain::Jungle,
        ] {
            assert!(g.supports(TerrainImprovement::Irrigation));
        }
        for g in [
            Terrain::Ocean,
            Terrain::Forest,
            Terrain::Hills,
            Terrain::Mountain,
            Terrain::Tundra,
        ] {
            assert!(!g.supports(TerrainImprovement::Irrigation));
        }
    }

    #[test]
    fn mine_supported_terrains() {
        for g in [Terrain::Hills, Terrain::Mountain, Terrain::Desert] {
            assert!(g.supports(TerrainImprovement::Mine));
        }
        for g in [
            Terrain::Ocean,
            Terrain::Grassland,
            Terrain::Plains,
            Terrain::Forest,
            Terrain::Tundra,
        ] {
            assert!(!g.supports(TerrainImprovement::Mine));
        }
    }

    #[test]
    fn road_supported_terrains() {
        assert!(!Terrain::Ocean.supports(TerrainImprovement::Road));
        for g in OPEN.iter().chain(DENSE.iter()).chain([&Terrain::Mountain]) {
            assert!(g.supports(TerrainImprovement::Road));
        }
    }

    #[test]
    fn special_resource_support_by_geography() {
        assert!(Terrain::Hills.supports_resource(SpecialResource::Coal));
        assert!(Terrain::Ocean.supports_resource(SpecialResource::Fish));
        assert!(Terrain::Forest.supports_resource(SpecialResource::Game));
        assert!(Terrain::Tundra.supports_resource(SpecialResource::Game));
        assert!(Terrain::Jungle.supports_resource(SpecialResource::Gems));
        assert!(Terrain::Mountain.supports_resource(SpecialResource::Gold));
        assert!(Terrain::Plains.supports_resource(SpecialResource::Horses));
        assert!(Terrain::Desert.supports_resource(SpecialResource::Oasis));
        assert!(Terrain::Swamp.supports_resource(SpecialResource::Oil));
    }

    #[test]
    fn special_resources_rejected_on_other_geography() {
        for g in [
            Terrain::Ocean,
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
            assert!(!g.supports_resource(SpecialResource::Coal) || matches!(g, Terrain::Hills));
            assert!(!g.supports_resource(SpecialResource::Fish) || matches!(g, Terrain::Ocean));
            assert!(
                !g.supports_resource(SpecialResource::Game)
                    || matches!(g, Terrain::Forest | Terrain::Tundra)
            );
            assert!(!g.supports_resource(SpecialResource::Gems) || matches!(g, Terrain::Jungle));
            assert!(!g.supports_resource(SpecialResource::Gold) || matches!(g, Terrain::Mountain));
            assert!(!g.supports_resource(SpecialResource::Horses) || matches!(g, Terrain::Plains));
            assert!(!g.supports_resource(SpecialResource::Oasis) || matches!(g, Terrain::Desert));
            assert!(!g.supports_resource(SpecialResource::Oil) || matches!(g, Terrain::Swamp));
        }
    }

    #[test]
    fn food_yields_per_geography() {
        assert_eq!(Terrain::Ocean.yields_food(), 2);
        assert_eq!(Terrain::Grassland.yields_food(), 2);
        assert_eq!(Terrain::Plains.yields_food(), 2);
        assert_eq!(Terrain::Forest.yields_food(), 1);
        assert_eq!(Terrain::Hills.yields_food(), 1);
        assert_eq!(Terrain::Mountain.yields_food(), 0);
        assert_eq!(Terrain::Desert.yields_food(), 0);
        assert_eq!(Terrain::Tundra.yields_food(), 1);
        assert_eq!(Terrain::Swamp.yields_food(), 1);
        assert_eq!(Terrain::Jungle.yields_food(), 1);
    }

    #[test]
    fn resources_yields_per_geography() {
        assert_eq!(Terrain::Ocean.yields_resources(), 0);
        assert_eq!(Terrain::Grassland.yields_resources(), 0);
        assert_eq!(Terrain::Plains.yields_resources(), 1);
        assert_eq!(Terrain::Forest.yields_resources(), 2);
        assert_eq!(Terrain::Hills.yields_resources(), 1);
        assert_eq!(Terrain::Mountain.yields_resources(), 1);
        assert_eq!(Terrain::Desert.yields_resources(), 1);
        assert_eq!(Terrain::Tundra.yields_resources(), 0);
        assert_eq!(Terrain::Swamp.yields_resources(), 0);
        assert_eq!(Terrain::Jungle.yields_resources(), 1);
    }

    #[test]
    fn trade_yields_per_geography() {
        assert_eq!(Terrain::Ocean.yields_trade(), 2);
        assert_eq!(Terrain::Grassland.yields_trade(), 1);
        assert_eq!(Terrain::Plains.yields_trade(), 0);
        assert_eq!(Terrain::Forest.yields_trade(), 0);
        assert_eq!(Terrain::Hills.yields_trade(), 0);
        assert_eq!(Terrain::Mountain.yields_trade(), 1);
        assert_eq!(Terrain::Desert.yields_trade(), 1);
        assert_eq!(Terrain::Tundra.yields_trade(), 1);
        assert_eq!(Terrain::Swamp.yields_trade(), 1);
        assert_eq!(Terrain::Jungle.yields_trade(), 0);
    }

    #[test]
    fn each_geography_renders_as_a_distinct_character() {
        let chars: Vec<char> = [
            Terrain::Ocean,
            Terrain::Grassland,
            Terrain::Plains,
            Terrain::Forest,
            Terrain::Hills,
            Terrain::Mountain,
            Terrain::Desert,
            Terrain::Tundra,
            Terrain::Swamp,
            Terrain::Jungle,
        ]
        .iter()
        .map(|geography| geography.as_char())
        .collect();
        assert_eq!(chars.len(), 10);
        let mut unique = chars.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 10, "every geography needs a distinct char");
    }
}
