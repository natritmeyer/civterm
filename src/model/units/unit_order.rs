use crate::model::geography::GeographyImprovement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitOrder {
    Idle,
    Fortified,
    Sentried,
    Improving(GeographyImprovement),
}
