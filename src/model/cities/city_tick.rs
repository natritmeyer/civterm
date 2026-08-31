use crate::model::cities::ProductionTarget;

/// Outcome of advancing a city one turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityTick {
    pub produced: u32,
    pub grew: bool,
    pub completed: Option<ProductionTarget>,
    pub starving: bool,
}
