use crate::model::advancements::Advancement;
use crate::model::cartography::Direction;
use crate::model::cities::{CityId, ProductionTarget};
use crate::model::civilizations::PlayerId;
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
    SetProductionTarget {
        city: CityId,
        target: ProductionTarget,
    },
    DeclareWar {
        opponent: PlayerId,
    },
    MakePeace {
        opponent: PlayerId,
    },
    SetResearchTarget {
        advancement: Advancement,
    },
    EndTurn,
}
