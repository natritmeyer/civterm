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
