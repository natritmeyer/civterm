#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainImprovement {
    Irrigation,
    Mine,
    Road,
}

impl TerrainImprovement {
    /// The improvement's name as it should appear to players.
    pub fn name(&self) -> &'static str {
        match self {
            TerrainImprovement::Irrigation => "Irrigation",
            TerrainImprovement::Mine => "Mine",
            TerrainImprovement::Road => "Road",
        }
    }

    /// How many turns a settler must spend working on a tile before the
    /// improvement lands: irrigation and roads take two turns, mines three.
    pub fn work_turns(&self) -> u8 {
        match self {
            TerrainImprovement::Irrigation => 2,
            TerrainImprovement::Mine => 3,
            TerrainImprovement::Road => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_improvement_has_a_distinct_name() {
        let names: Vec<&str> = [
            TerrainImprovement::Irrigation,
            TerrainImprovement::Mine,
            TerrainImprovement::Road,
        ]
        .iter()
        .map(|improvement| improvement.name())
        .collect();
        assert_eq!(
            names,
            vec!["Irrigation", "Mine", "Road"],
            "improvement names are stable display labels"
        );
    }

    #[test]
    fn work_turns_differ_by_improvement() {
        assert_eq!(TerrainImprovement::Irrigation.work_turns(), 2);
        assert_eq!(TerrainImprovement::Road.work_turns(), 2);
        assert_eq!(TerrainImprovement::Mine.work_turns(), 3);
    }
}
