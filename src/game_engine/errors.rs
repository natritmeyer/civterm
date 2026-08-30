use crate::model::cartography::Location;
use crate::model::units::UnitId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    NoSuchUnit(UnitId),
    NoMovesRemaining(UnitId),
    CannotMoveThere,
    TerrainTooDifficult(UnitId),
    CannotCrossLandSeaBorder(UnitId),
}

impl MoveError {
    pub fn message(&self) -> String {
        match self {
            MoveError::NoSuchUnit(_) => "No such unit".to_string(),
            MoveError::NoMovesRemaining(unit) => format!("Unit {} has no moves left", unit.index()),
            MoveError::CannotMoveThere => "Cannot move there".to_string(),
            MoveError::TerrainTooDifficult(_) => "Terrain too difficult".to_string(),
            MoveError::CannotCrossLandSeaBorder(unit) => {
                format!("Unit {} cannot cross land/sea border", unit.index())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettleError {
    NoSuchUnit(UnitId),
    NotASettler(UnitId),
    LandRequired(UnitId),
    CityAlreadyHere(Location),
}

impl SettleError {
    pub fn message(&self) -> String {
        match self {
            SettleError::NoSuchUnit(_) => "No such unit".to_string(),
            SettleError::NotASettler(unit) => {
                format!("Unit {} cannot found a city", unit.index())
            }
            SettleError::LandRequired(unit) => {
                format!("Unit {} must be on land to found a city", unit.index())
            }
            SettleError::CityAlreadyHere(_) => "A city already occupies that tile".to_string(),
        }
    }
}
