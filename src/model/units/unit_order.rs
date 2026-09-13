use crate::model::geography::TerrainImprovement;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitOrder {
    Idle,
    Fortified,
    Sentried,
    Improving(TerrainImprovement),
}
