use crate::model::geography::TerrainImprovement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitOrder {
    Idle,
    Fortified,
    Sentried,
    Improving(TerrainImprovement),
}
