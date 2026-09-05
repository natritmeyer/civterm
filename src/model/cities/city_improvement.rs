use crate::model::advancements::Advancement;
use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum CityImprovement {
    Aqueduct,
    Bank,
    Barracks,
    Cathedral,
    CityWalls,
    Colosseum,
    Courthouse,
    Granary,
    Library,
    Marketplace,
    Palace,
    Temple,
    University,
}

impl CityImprovement {
    /// The improvement's name as it should appear to players.
    pub fn name(&self) -> &'static str {
        match self {
            CityImprovement::Aqueduct => "Aqueduct",
            CityImprovement::Bank => "Bank",
            CityImprovement::Barracks => "Barracks",
            CityImprovement::Cathedral => "Cathedral",
            CityImprovement::CityWalls => "City Walls",
            CityImprovement::Colosseum => "Colosseum",
            CityImprovement::Courthouse => "Courthouse",
            CityImprovement::Granary => "Granary",
            CityImprovement::Library => "Library",
            CityImprovement::Marketplace => "Marketplace",
            CityImprovement::Palace => "Palace",
            CityImprovement::Temple => "Temple",
            CityImprovement::University => "University",
        }
    }

    pub fn required_advancement(&self) -> Option<Advancement> {
        match self {
            CityImprovement::Aqueduct => Some(Advancement::Construction),
            CityImprovement::Bank => Some(Advancement::Banking),
            CityImprovement::Barracks => None,
            CityImprovement::Cathedral => Some(Advancement::Religion),
            CityImprovement::CityWalls => Some(Advancement::Masonry),
            CityImprovement::Colosseum => Some(Advancement::Construction),
            CityImprovement::Courthouse => Some(Advancement::CodeOfLaws),
            CityImprovement::Granary => Some(Advancement::Pottery),
            CityImprovement::Library => Some(Advancement::Writing),
            CityImprovement::Marketplace => Some(Advancement::Currency),
            CityImprovement::Palace => Some(Advancement::Masonry),
            CityImprovement::Temple => Some(Advancement::CeremonialBurial),
            CityImprovement::University => Some(Advancement::University),
        }
    }

    pub fn resource_cost(&self) -> u32 {
        match self {
            CityImprovement::Library
            | CityImprovement::Granary
            | CityImprovement::Marketplace
            | CityImprovement::Temple
            | CityImprovement::Colosseum
            | CityImprovement::Cathedral
            | CityImprovement::Courthouse
            | CityImprovement::Bank
            | CityImprovement::University => 20,
            CityImprovement::CityWalls | CityImprovement::Barracks | CityImprovement::Aqueduct => {
                15
            }
            CityImprovement::Palace => 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn improvements_require_their_advance() {
        assert_eq!(
            CityImprovement::Aqueduct.required_advancement(),
            Some(Advancement::Construction)
        );
        assert_eq!(
            CityImprovement::Bank.required_advancement(),
            Some(Advancement::Banking)
        );
        assert_eq!(
            CityImprovement::Cathedral.required_advancement(),
            Some(Advancement::Religion)
        );
        assert_eq!(
            CityImprovement::CityWalls.required_advancement(),
            Some(Advancement::Masonry)
        );
        assert_eq!(
            CityImprovement::Granary.required_advancement(),
            Some(Advancement::Pottery)
        );
        assert_eq!(
            CityImprovement::Library.required_advancement(),
            Some(Advancement::Writing)
        );
        assert_eq!(
            CityImprovement::Temple.required_advancement(),
            Some(Advancement::CeremonialBurial)
        );
        assert_eq!(
            CityImprovement::University.required_advancement(),
            Some(Advancement::University)
        );
    }

    #[test]
    fn improvements_without_a_required_advancement() {
        assert_eq!(CityImprovement::Barracks.required_advancement(), None);
    }

    #[test]
    fn improvement_production_costs() {
        assert_eq!(CityImprovement::CityWalls.resource_cost(), 15);
        assert_eq!(CityImprovement::Barracks.resource_cost(), 15);
        assert_eq!(CityImprovement::Aqueduct.resource_cost(), 15);
        assert_eq!(CityImprovement::Library.resource_cost(), 20);
        assert_eq!(CityImprovement::Temple.resource_cost(), 20);
        assert_eq!(CityImprovement::Palace.resource_cost(), 30);
    }
}
