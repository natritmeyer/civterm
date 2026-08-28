use crate::model::cartography::Location;
use crate::model::units::UnitClass;

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub unit_class: UnitClass,
    pub location: Location,
}

impl Unit {
    pub fn new(unit_class: UnitClass, location: Location) -> Self {
        Unit {
            unit_class,
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_is_created_with_class_and_location() {
        let location = Location::new(1, 7);
        let unit = Unit::new(UnitClass::Settler, location);
        assert_eq!(unit.unit_class, UnitClass::Settler);
        assert_eq!(unit.location, location);
    }
}
