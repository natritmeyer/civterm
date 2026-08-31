#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitClass {
    Settler,
    Militia,
    Phalanx,
    Legion,
    Cavalry,
    Chariot,
    Knight,
    Catapult,
    Diplomat,
    Caravan,
    Trireme,
    Sail,
    Frigate,
}

impl UnitClass {
    pub fn attack(&self) -> u8 {
        match self {
            UnitClass::Settler | UnitClass::Caravan | UnitClass::Diplomat => 0,
            UnitClass::Militia | UnitClass::Phalanx | UnitClass::Trireme | UnitClass::Sail => 1,
            UnitClass::Cavalry | UnitClass::Frigate => 2,
            UnitClass::Legion => 3,
            UnitClass::Chariot | UnitClass::Knight => 4,
            UnitClass::Catapult => 6,
        }
    }

    pub fn defence(&self) -> u8 {
        match self {
            UnitClass::Diplomat | UnitClass::Trireme => 0,
            UnitClass::Settler
            | UnitClass::Militia
            | UnitClass::Legion
            | UnitClass::Cavalry
            | UnitClass::Chariot
            | UnitClass::Caravan
            | UnitClass::Sail
            | UnitClass::Catapult => 1,
            UnitClass::Phalanx | UnitClass::Knight | UnitClass::Frigate => 2,
        }
    }

    pub fn can_found_city(&self) -> bool {
        matches!(self, UnitClass::Settler)
    }

    pub fn can_travel_water(&self) -> bool {
        matches!(
            self,
            UnitClass::Trireme | UnitClass::Sail | UnitClass::Frigate
        )
    }

    pub fn moves(&self) -> u8 {
        match self {
            UnitClass::Settler
            | UnitClass::Militia
            | UnitClass::Phalanx
            | UnitClass::Legion
            | UnitClass::Catapult
            | UnitClass::Diplomat
            | UnitClass::Caravan => 1,
            UnitClass::Cavalry | UnitClass::Chariot | UnitClass::Knight => 3,
            UnitClass::Trireme | UnitClass::Sail | UnitClass::Frigate => 3,
        }
    }

    pub fn resource_cost(&self) -> u32 {
        match self {
            UnitClass::Militia => 10,
            UnitClass::Phalanx | UnitClass::Legion | UnitClass::Chariot => 15,
            UnitClass::Knight
            | UnitClass::Cavalry
            | UnitClass::Catapult
            | UnitClass::Diplomat
            | UnitClass::Sail => 20,
            UnitClass::Caravan | UnitClass::Trireme => 30,
            UnitClass::Frigate => 40,
            UnitClass::Settler => 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn land_unit_stats() {
        assert_eq!(UnitClass::Settler.attack(), 0);
        assert_eq!(UnitClass::Settler.defence(), 1);
        assert_eq!(UnitClass::Militia.attack(), 1);
        assert_eq!(UnitClass::Militia.defence(), 1);
        assert_eq!(UnitClass::Phalanx.attack(), 1);
        assert_eq!(UnitClass::Phalanx.defence(), 2);
        assert_eq!(UnitClass::Legion.attack(), 3);
        assert_eq!(UnitClass::Legion.defence(), 1);
        assert_eq!(UnitClass::Cavalry.attack(), 2);
        assert_eq!(UnitClass::Cavalry.defence(), 1);
        assert_eq!(UnitClass::Chariot.attack(), 4);
        assert_eq!(UnitClass::Chariot.defence(), 1);
        assert_eq!(UnitClass::Knight.attack(), 4);
        assert_eq!(UnitClass::Knight.defence(), 2);
        assert_eq!(UnitClass::Catapult.attack(), 6);
        assert_eq!(UnitClass::Catapult.defence(), 1);
        assert_eq!(UnitClass::Diplomat.attack(), 0);
        assert_eq!(UnitClass::Diplomat.defence(), 0);
        assert_eq!(UnitClass::Caravan.attack(), 0);
        assert_eq!(UnitClass::Caravan.defence(), 1);
    }

    #[test]
    fn sea_unit_stats() {
        assert_eq!(UnitClass::Trireme.attack(), 1);
        assert_eq!(UnitClass::Trireme.defence(), 0);
        assert_eq!(UnitClass::Sail.attack(), 1);
        assert_eq!(UnitClass::Sail.defence(), 1);
        assert_eq!(UnitClass::Frigate.attack(), 2);
        assert_eq!(UnitClass::Frigate.defence(), 2);
    }

    #[test]
    fn unit_movement_rates() {
        for class in [
            UnitClass::Settler,
            UnitClass::Militia,
            UnitClass::Phalanx,
            UnitClass::Legion,
            UnitClass::Catapult,
            UnitClass::Diplomat,
            UnitClass::Caravan,
        ] {
            assert_eq!(class.moves(), 1);
        }
        for class in [
            UnitClass::Cavalry,
            UnitClass::Chariot,
            UnitClass::Knight,
            UnitClass::Trireme,
            UnitClass::Sail,
            UnitClass::Frigate,
        ] {
            assert_eq!(class.moves(), 3);
        }
    }

    #[test]
    fn unit_production_costs() {
        assert_eq!(UnitClass::Militia.resource_cost(), 10);
        assert_eq!(UnitClass::Phalanx.resource_cost(), 15);
        assert_eq!(UnitClass::Legion.resource_cost(), 15);
        assert_eq!(UnitClass::Chariot.resource_cost(), 15);
        assert_eq!(UnitClass::Knight.resource_cost(), 20);
        assert_eq!(UnitClass::Cavalry.resource_cost(), 20);
        assert_eq!(UnitClass::Catapult.resource_cost(), 20);
        assert_eq!(UnitClass::Diplomat.resource_cost(), 20);
        assert_eq!(UnitClass::Sail.resource_cost(), 20);
        assert_eq!(UnitClass::Caravan.resource_cost(), 30);
        assert_eq!(UnitClass::Trireme.resource_cost(), 30);
        assert_eq!(UnitClass::Frigate.resource_cost(), 40);
        assert_eq!(UnitClass::Settler.resource_cost(), 60);
    }

    #[test]
    fn only_settlers_can_found_cities() {
        assert!(UnitClass::Settler.can_found_city());
        for class in [
            UnitClass::Militia,
            UnitClass::Phalanx,
            UnitClass::Legion,
            UnitClass::Cavalry,
            UnitClass::Chariot,
            UnitClass::Knight,
            UnitClass::Catapult,
            UnitClass::Diplomat,
            UnitClass::Caravan,
            UnitClass::Trireme,
            UnitClass::Sail,
            UnitClass::Frigate,
        ] {
            assert!(!class.can_found_city());
        }
    }

    #[test]
    fn only_naval_units_can_travel_water() {
        for class in [UnitClass::Trireme, UnitClass::Sail, UnitClass::Frigate] {
            assert!(class.can_travel_water());
        }
        for class in [
            UnitClass::Settler,
            UnitClass::Militia,
            UnitClass::Phalanx,
            UnitClass::Legion,
            UnitClass::Cavalry,
            UnitClass::Chariot,
            UnitClass::Knight,
            UnitClass::Catapult,
            UnitClass::Diplomat,
            UnitClass::Caravan,
        ] {
            assert!(!class.can_travel_water());
        }
    }
}
