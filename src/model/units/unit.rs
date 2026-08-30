use crate::model::cartography::Location;
use crate::model::cities::CityId;
use crate::model::civilizations::PlayerId;
use crate::model::geography::GeographyImprovement;
use crate::model::units::{UnitClass, UnitId, UnitOrder};

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub unit_class: UnitClass,
    pub location: Location,
    id: UnitId,
    owner: PlayerId,
    home_city: CityId,
    order: UnitOrder,
    veteran: bool,
    moves_remaining: u8,
}

impl Unit {
    pub fn new(
        unit_class: UnitClass,
        location: Location,
        owner: PlayerId,
        home_city: CityId,
        id: UnitId,
    ) -> Self {
        Unit {
            unit_class,
            location,
            id,
            owner,
            home_city,
            order: UnitOrder::Idle,
            veteran: false,
            moves_remaining: unit_class.moves(),
        }
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn home_city(&self) -> CityId {
        self.home_city
    }

    pub fn order(&self) -> UnitOrder {
        self.order
    }

    pub fn moves_remaining(&self) -> u8 {
        self.moves_remaining
    }

    pub fn spend_moves(&mut self, amount: u8) {
        self.moves_remaining = self.moves_remaining.saturating_sub(amount);
    }

    pub fn spend_turn(&mut self) {
        self.moves_remaining = 0;
    }

    pub fn restore_moves(&mut self) {
        self.moves_remaining = self.unit_class.moves();
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
    fn unit_is_created_with_class_location_owner_and_home_city() {
        let location = Location::new(1, 7);
        let home_city = CityId::new(4);
        let id = UnitId::new(2);
        let unit = Unit::new(
            UnitClass::Settler,
            location,
            PlayerId::new(0),
            home_city,
            id,
        );
        assert_eq!(unit.unit_class, UnitClass::Settler);
        assert_eq!(unit.location, location);
        assert_eq!(unit.owner(), PlayerId::new(0));
        assert_eq!(unit.home_city(), home_city);
        assert_eq!(unit.id(), id);
    }

    #[test]
    fn unit_starts_idle() {
        let unit = Unit::new(
            UnitClass::Settler,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        assert_eq!(unit.order(), UnitOrder::Idle);
    }

    #[test]
    fn unit_can_be_fortified() {
        let mut unit = Unit::new(
            UnitClass::Legion,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.fortify();
        assert_eq!(unit.order(), UnitOrder::Fortified);
    }

    #[test]
    fn unit_can_be_sentried() {
        let mut unit = Unit::new(
            UnitClass::Legion,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.sentry();
        assert_eq!(unit.order(), UnitOrder::Sentried);
    }

    #[test]
    fn unit_can_work_an_improvement() {
        let mut unit = Unit::new(
            UnitClass::Settler,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
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
        let mut unit = Unit::new(
            UnitClass::Settler,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.fortify();
        unit.work(GeographyImprovement::Mine);
        assert_eq!(
            unit.order(),
            UnitOrder::Improving(GeographyImprovement::Mine)
        );
    }

    #[test]
    fn order_can_be_cancelled() {
        let mut unit = Unit::new(
            UnitClass::Settler,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.fortify();
        unit.cancel_order();
        assert_eq!(unit.order(), UnitOrder::Idle);
    }

    #[test]
    fn unit_starts_as_regular() {
        let unit = Unit::new(
            UnitClass::Legion,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        assert!(!unit.is_veteran());
    }

    #[test]
    fn unit_starts_with_moves_available() {
        let unit = Unit::new(
            UnitClass::Chariot,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        assert_eq!(unit.moves_remaining(), 3);
    }

    #[test]
    fn moves_are_spent_and_restored() {
        let mut unit = Unit::new(
            UnitClass::Chariot,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.spend_moves(2);
        assert_eq!(unit.moves_remaining(), 1);
        unit.spend_moves(2);
        assert_eq!(unit.moves_remaining(), 0);
        unit.restore_moves();
        assert_eq!(unit.moves_remaining(), 3);
    }

    #[test]
    fn spending_a_turn_uses_up_all_moves() {
        let mut unit = Unit::new(
            UnitClass::Chariot,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.spend_turn();
        assert_eq!(unit.moves_remaining(), 0);
    }

    #[test]
    fn unit_is_promoted_to_veteran() {
        let mut unit = Unit::new(
            UnitClass::Legion,
            Location::new(0, 0),
            PlayerId::new(0),
            CityId::new(0),
            UnitId::new(0),
        );
        unit.promote();
        assert!(unit.is_veteran());
    }
}
