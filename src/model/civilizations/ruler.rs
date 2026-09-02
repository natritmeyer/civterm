#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ruler {
    AbrahamLincoln,
    Montezuma,
    Hammurabi,
    MaoZedong,
    Ramses,
    QueenElizabethI,
    Napoleon,
    FrederickTheGreat,
    Alexander,
    Gandhi,
    GenghisKhan,
    JuliusCaesar,
    Stalin,
    Shaka,
}

impl Ruler {
    pub fn display_name(&self) -> &'static str {
        match self {
            Ruler::AbrahamLincoln => "Abraham Lincoln",
            Ruler::Montezuma => "Montezuma",
            Ruler::Hammurabi => "Hammurabi",
            Ruler::MaoZedong => "Mao Zedong",
            Ruler::Ramses => "Ramses",
            Ruler::QueenElizabethI => "Queen Elizabeth I",
            Ruler::Napoleon => "Napoleon",
            Ruler::FrederickTheGreat => "Frederick the Great",
            Ruler::Alexander => "Alexander",
            Ruler::Gandhi => "Gandhi",
            Ruler::GenghisKhan => "Genghis Khan",
            Ruler::JuliusCaesar => "Julius Caesar",
            Ruler::Stalin => "Stalin",
            Ruler::Shaka => "Shaka",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::civilizations::Civilization;
    use strum::IntoEnumIterator;

    #[test]
    fn display_name_for_each_ruler() {
        assert_eq!(Ruler::AbrahamLincoln.display_name(), "Abraham Lincoln");
        assert_eq!(Ruler::Montezuma.display_name(), "Montezuma");
        assert_eq!(Ruler::Hammurabi.display_name(), "Hammurabi");
        assert_eq!(Ruler::MaoZedong.display_name(), "Mao Zedong");
        assert_eq!(Ruler::Ramses.display_name(), "Ramses");
        assert_eq!(Ruler::QueenElizabethI.display_name(), "Queen Elizabeth I");
        assert_eq!(Ruler::Napoleon.display_name(), "Napoleon");
        assert_eq!(
            Ruler::FrederickTheGreat.display_name(),
            "Frederick the Great"
        );
        assert_eq!(Ruler::Alexander.display_name(), "Alexander");
        assert_eq!(Ruler::Gandhi.display_name(), "Gandhi");
        assert_eq!(Ruler::GenghisKhan.display_name(), "Genghis Khan");
        assert_eq!(Ruler::JuliusCaesar.display_name(), "Julius Caesar");
        assert_eq!(Ruler::Stalin.display_name(), "Stalin");
        assert_eq!(Ruler::Shaka.display_name(), "Shaka");
    }

    #[test]
    fn every_ruler_belongs_to_a_civilization() {
        assert_eq!(Civilization::iter().count(), 14);
        let rulers: Vec<Ruler> = Civilization::iter()
            .map(|civilization| civilization.ruler())
            .collect();
        assert_eq!(rulers.len(), 14);
        for ruler in rulers {
            assert!(!ruler.display_name().is_empty());
        }
    }

    #[test]
    fn display_names_are_distinct() {
        let mut names: Vec<&str> = Civilization::iter()
            .map(|civilization| civilization.ruler().display_name())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 14, "no two rulers share a display name");
    }
}
