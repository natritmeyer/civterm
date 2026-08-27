use crate::model::geography_improvement::GeographyImprovement;
use crate::model::movement_category::MovementCategory;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geography {
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

impl Geography {
    pub fn movement_class(&self) -> MovementCategory {
        match self {
            Geography::Ocean
            | Geography::Grassland
            | Geography::Plains
            | Geography::Desert
            | Geography::Tundra => MovementCategory::Open,
            Geography::Forest | Geography::Hills | Geography::Swamp | Geography::Jungle => {
                MovementCategory::Dense
            }
            Geography::Mountain => MovementCategory::Mountain,
        }
    }

    pub fn movement_cost(&self) -> u8 {
        self.movement_class().movement_cost()
    }

    pub fn is_water(&self) -> bool {
        matches!(self, Geography::Ocean)
    }

    pub fn is_land(&self) -> bool {
        !self.is_water()
    }

    pub fn supports(&self, improvement: GeographyImprovement) -> bool {
        match improvement {
            GeographyImprovement::Irrigation => matches!(
                self,
                Geography::Grassland
                    | Geography::Plains
                    | Geography::Desert
                    | Geography::Swamp
                    | Geography::Jungle
            ),
            GeographyImprovement::Mine => matches!(
                self,
                Geography::Hills | Geography::Mountain | Geography::Desert
            ),
            GeographyImprovement::Road => self.is_land(),
        }
    }

    pub fn yields_food(&self) -> u8 {
        match self {
            Geography::Ocean | Geography::Grassland | Geography::Plains => 2,
            Geography::Forest
            | Geography::Hills
            | Geography::Tundra
            | Geography::Swamp
            | Geography::Jungle => 1,
            Geography::Mountain | Geography::Desert => 0,
        }
    }

    pub fn yields_resources(&self) -> u8 {
        match self {
            Geography::Forest => 2,
            Geography::Plains
            | Geography::Hills
            | Geography::Mountain
            | Geography::Desert
            | Geography::Jungle => 1,
            Geography::Ocean | Geography::Grassland | Geography::Tundra | Geography::Swamp => 0,
        }
    }

    pub fn yields_trade(&self) -> u8 {
        match self {
            Geography::Ocean => 2,
            Geography::Grassland
            | Geography::Mountain
            | Geography::Desert
            | Geography::Tundra
            | Geography::Swamp => 1,
            Geography::Plains | Geography::Forest | Geography::Hills | Geography::Jungle => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATER: [Geography; 1] = [Geography::Ocean];

    const OPEN: [Geography; 4] = [
        Geography::Grassland,
        Geography::Plains,
        Geography::Desert,
        Geography::Tundra,
    ];

    const DENSE: [Geography; 4] = [
        Geography::Forest,
        Geography::Hills,
        Geography::Swamp,
        Geography::Jungle,
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
            Geography::Mountain.movement_class(),
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
        assert_eq!(Geography::Mountain.movement_cost(), 3);
    }

    #[test]
    fn only_ocean_is_water() {
        for g in WATER {
            assert!(g.is_water());
            assert!(!g.is_land());
        }
        for g in OPEN
            .iter()
            .chain(DENSE.iter())
            .chain([&Geography::Mountain])
        {
            assert!(!g.is_water());
            assert!(g.is_land());
        }
    }

    #[test]
    fn irrigation_supported_terrains() {
        for g in [
            Geography::Grassland,
            Geography::Plains,
            Geography::Desert,
            Geography::Swamp,
            Geography::Jungle,
        ] {
            assert!(g.supports(GeographyImprovement::Irrigation));
        }
        for g in [
            Geography::Ocean,
            Geography::Forest,
            Geography::Hills,
            Geography::Mountain,
            Geography::Tundra,
        ] {
            assert!(!g.supports(GeographyImprovement::Irrigation));
        }
    }

    #[test]
    fn mine_supported_terrains() {
        for g in [Geography::Hills, Geography::Mountain, Geography::Desert] {
            assert!(g.supports(GeographyImprovement::Mine));
        }
        for g in [
            Geography::Ocean,
            Geography::Grassland,
            Geography::Plains,
            Geography::Forest,
            Geography::Tundra,
        ] {
            assert!(!g.supports(GeographyImprovement::Mine));
        }
    }

    #[test]
    fn road_supported_terrains() {
        assert!(!Geography::Ocean.supports(GeographyImprovement::Road));
        for g in OPEN
            .iter()
            .chain(DENSE.iter())
            .chain([&Geography::Mountain])
        {
            assert!(g.supports(GeographyImprovement::Road));
        }
    }

    #[test]
    fn food_yields_per_geography() {
        assert_eq!(Geography::Ocean.yields_food(), 2);
        assert_eq!(Geography::Grassland.yields_food(), 2);
        assert_eq!(Geography::Plains.yields_food(), 2);
        assert_eq!(Geography::Forest.yields_food(), 1);
        assert_eq!(Geography::Hills.yields_food(), 1);
        assert_eq!(Geography::Mountain.yields_food(), 0);
        assert_eq!(Geography::Desert.yields_food(), 0);
        assert_eq!(Geography::Tundra.yields_food(), 1);
        assert_eq!(Geography::Swamp.yields_food(), 1);
        assert_eq!(Geography::Jungle.yields_food(), 1);
    }

    #[test]
    fn resources_yields_per_geography() {
        assert_eq!(Geography::Ocean.yields_resources(), 0);
        assert_eq!(Geography::Grassland.yields_resources(), 0);
        assert_eq!(Geography::Plains.yields_resources(), 1);
        assert_eq!(Geography::Forest.yields_resources(), 2);
        assert_eq!(Geography::Hills.yields_resources(), 1);
        assert_eq!(Geography::Mountain.yields_resources(), 1);
        assert_eq!(Geography::Desert.yields_resources(), 1);
        assert_eq!(Geography::Tundra.yields_resources(), 0);
        assert_eq!(Geography::Swamp.yields_resources(), 0);
        assert_eq!(Geography::Jungle.yields_resources(), 1);
    }

    #[test]
    fn trade_yields_per_geography() {
        assert_eq!(Geography::Ocean.yields_trade(), 2);
        assert_eq!(Geography::Grassland.yields_trade(), 1);
        assert_eq!(Geography::Plains.yields_trade(), 0);
        assert_eq!(Geography::Forest.yields_trade(), 0);
        assert_eq!(Geography::Hills.yields_trade(), 0);
        assert_eq!(Geography::Mountain.yields_trade(), 1);
        assert_eq!(Geography::Desert.yields_trade(), 1);
        assert_eq!(Geography::Tundra.yields_trade(), 1);
        assert_eq!(Geography::Swamp.yields_trade(), 1);
        assert_eq!(Geography::Jungle.yields_trade(), 0);
    }
}
