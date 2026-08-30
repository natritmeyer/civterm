use crate::model::cartography::Direction;
use crate::model::geography::GeographyImprovement;
use crate::model::units::UnitId;

#[derive(Debug)]
pub enum Command {
    Move {
        unit: UnitId,
        direction: Direction,
    },
    Fortify {
        unit: UnitId,
    },
    Sentry {
        unit: UnitId,
    },
    Work {
        unit: UnitId,
        improvement: GeographyImprovement,
    },
    CancelOrder {
        unit: UnitId,
    },
    FoundCity {
        unit: UnitId,
        name: String,
    },
    EndTurn,
}
