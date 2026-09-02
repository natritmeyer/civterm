use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn difficulty_has_three_levels_in_order() {
        let levels: Vec<Difficulty> = Difficulty::iter().collect();
        assert_eq!(
            levels,
            vec![Difficulty::Easy, Difficulty::Normal, Difficulty::Hard]
        );
    }

    #[test]
    fn each_difficulty_has_a_display_name() {
        assert_eq!(Difficulty::Easy.display_name(), "Easy");
        assert_eq!(Difficulty::Normal.display_name(), "Normal");
        assert_eq!(Difficulty::Hard.display_name(), "Hard");
    }
}
