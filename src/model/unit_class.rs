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
}
