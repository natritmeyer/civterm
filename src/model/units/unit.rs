use crate::model::cartography::Location;
use crate::model::geography::GeographyImprovement;
use crate::model::units::UnitClass;
use crate::model::units::UnitOrder;

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub unit_class: UnitClass,
    pub location: Location,
    order: UnitOrder,
    veteran: bool,
}

impl Unit {
    pub fn new(unit_class: UnitClass, location: Location) -> Self {
        Unit {
            unit_class,
            location,
            order: UnitOrder::Idle,
            veteran: false,
        }
    }

    pub fn order(&self) -> UnitOrder {
        self.order
    }

    pub fn fortify(&mut self) {
        self.order = UnitOrder::Fortified;
    }

    pub fn sentry(&mut self) {
        self.order = UnitOrder::Sentried;
    }

    pub fn work(&mut self, improvement: GeographyImprovement) {
        self.order = UnitOrder::Improving(improvement);
    }

    pub fn cancel_order(&mut self) {
        self.order = UnitOrder::Idle;
    }

    pub fn is_veteran(&self) -> bool {
        self.veteran
    }

    pub fn promote(&mut self) {
        self.veteran = true;
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

    #[test]
    fn unit_starts_idle() {
        let unit = Unit::new(UnitClass::Settler, Location::new(0, 0));
        assert_eq!(unit.order(), UnitOrder::Idle);
    }

    #[test]
    fn unit_can_be_fortified() {
        let mut unit = Unit::new(UnitClass::Legion, Location::new(0, 0));
        unit.fortify();
        assert_eq!(unit.order(), UnitOrder::Fortified);
    }

    #[test]
    fn unit_can_be_sentried() {
        let mut unit = Unit::new(UnitClass::Legion, Location::new(0, 0));
        unit.sentry();
        assert_eq!(unit.order(), UnitOrder::Sentried);
    }

    #[test]
    fn unit_can_work_an_improvement() {
        let mut unit = Unit::new(UnitClass::Settler, Location::new(0, 0));
        unit.work(GeographyImprovement::Road);
        assert_eq!(
            unit.order(),
            UnitOrder::Improving(GeographyImprovement::Road)
        );
        unit.work(GeographyImprovement::Irrigation);
        assert_eq!(
            unit.order(),
            UnitOrder::Improving(GeographyImprovement::Irrigation)
        );
    }

    #[test]
    fn working_replaces_fortify() {
        let mut unit = Unit::new(UnitClass::Settler, Location::new(0, 0));
        unit.fortify();
        unit.work(GeographyImprovement::Mine);
        assert_eq!(
            unit.order(),
            UnitOrder::Improving(GeographyImprovement::Mine)
        );
    }

    #[test]
    fn order_can_be_cancelled() {
        let mut unit = Unit::new(UnitClass::Settler, Location::new(0, 0));
        unit.fortify();
        unit.cancel_order();
        assert_eq!(unit.order(), UnitOrder::Idle);
    }

    #[test]
    fn unit_starts_as_regular() {
        let unit = Unit::new(UnitClass::Legion, Location::new(0, 0));
        assert!(!unit.is_veteran());
    }

    #[test]
    fn unit_is_promoted_to_veteran() {
        let mut unit = Unit::new(UnitClass::Legion, Location::new(0, 0));
        unit.promote();
        assert!(unit.is_veteran());
    }
}
