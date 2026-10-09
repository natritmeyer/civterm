use crate::model::cities::ProductionTarget;

/// Outcome of advancing a city one turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityTick {
    pub produced: u32,
    pub grew: bool,
    pub completed: Option<ProductionTarget>,
    /// The city had an empty granary and a food deficit this turn. True even
    /// for a size-one city, which needs the warning but cannot lose its last
    /// citizen.
    pub starving: bool,
    /// A citizen was actually lost to starvation this turn. This is the flag a
    /// window may report a population loss from: it is never set for the last
    /// citizen, so a lie ("lost 10,000 population") cannot be told about a
    /// city that kept everyone.
    pub lost_citizen: bool,
}
