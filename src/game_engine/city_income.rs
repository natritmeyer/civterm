use crate::model::geography::SpecialResource;

/// What a city harvests in a single turn from its centre and worked tiles.
pub struct CityIncome {
    pub food: u32,
    pub resources: u32,
    pub trade: u32,
    pub gold: u32,
    pub research: u32,
    /// The distinct special resources being worked by this city.
    pub special_resources: Vec<SpecialResource>,
}
