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
}
