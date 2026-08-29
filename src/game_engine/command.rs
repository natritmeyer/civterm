use crate::model::cartography::Direction;
use crate::model::geography::GeographyImprovement;

#[derive(Debug)]
pub enum Command {
    Move {
        unit: usize,
        direction: Direction,
    },
    Fortify {
        unit: usize,
    },
    Sentry {
        unit: usize,
    },
    Work {
        unit: usize,
        improvement: GeographyImprovement,
    },
    CancelOrder {
        unit: usize,
    },
    EndTurn,
}
