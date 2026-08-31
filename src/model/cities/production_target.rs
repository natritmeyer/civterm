use crate::model::cities::CityImprovement;
use crate::model::units::UnitClass;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductionTarget {
    Unit(UnitClass),
    Improvement(CityImprovement),
}

impl ProductionTarget {
    pub fn resource_cost(&self) -> u32 {
        match self {
            ProductionTarget::Unit(unit_class) => unit_class.resource_cost(),
            ProductionTarget::Improvement(improvement) => improvement.resource_cost(),
        }
    }
}
