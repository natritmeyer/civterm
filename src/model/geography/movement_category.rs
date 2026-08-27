#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementCategory {
    Open,
    Dense,
    Mountain,
}

impl MovementCategory {
    pub fn movement_cost(&self) -> u8 {
        match self {
            MovementCategory::Open => 1,
            MovementCategory::Dense => 2,
            MovementCategory::Mountain => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_category_costs() {
        assert_eq!(MovementCategory::Open.movement_cost(), 1);
        assert_eq!(MovementCategory::Dense.movement_cost(), 2);
        assert_eq!(MovementCategory::Mountain.movement_cost(), 3);
    }
}
