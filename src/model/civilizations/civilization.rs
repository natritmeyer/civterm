use crate::model::civilizations::Ruler;
use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum Civilization {
    American,
    Aztec,
    Babylonian,
    Chinese,
    Egyptian,
    English,
    French,
    German,
    Greek,
    Indian,
    Mongol,
    Roman,
    Russian,
    Zulu,
}

impl Civilization {
    pub fn display_name(&self) -> &'static str {
        match self {
            Civilization::American => "American",
            Civilization::Aztec => "Aztec",
            Civilization::Babylonian => "Babylonian",
            Civilization::Chinese => "Chinese",
            Civilization::Egyptian => "Egyptian",
            Civilization::English => "English",
            Civilization::French => "French",
            Civilization::German => "German",
            Civilization::Greek => "Greek",
            Civilization::Indian => "Indian",
            Civilization::Mongol => "Mongol",
            Civilization::Roman => "Roman",
            Civilization::Russian => "Russian",
            Civilization::Zulu => "Zulu",
        }
    }

    pub fn motto(&self) -> &'static str {
        match self {
            Civilization::American => "Life, liberty, and happiness",
            Civilization::Aztec => "Empire of the sun",
            Civilization::Babylonian => "By the laws, the land stands",
            Civilization::Chinese => "The people, nothing but the people",
            Civilization::Egyptian => "Gifts of the Nile",
            Civilization::English => "Might such as has never marched",
            Civilization::French => "The destiny of France",
            Civilization::German => "The first servant of the state",
            Civilization::Greek => "Strength through knowledge",
            Civilization::Indian => "Truth alone triumphs",
            Civilization::Mongol => "The punishment of God",
            Civilization::Roman => "Veni, vidi, vici",
            Civilization::Russian => "The iron fist of the people",
            Civilization::Zulu => "The spear of the nation",
        }
    }

    pub fn ruler(&self) -> Ruler {
        match self {
            Civilization::American => Ruler::AbrahamLincoln,
            Civilization::Aztec => Ruler::Montezuma,
            Civilization::Babylonian => Ruler::Hammurabi,
            Civilization::Chinese => Ruler::MaoZedong,
            Civilization::Egyptian => Ruler::Ramses,
            Civilization::English => Ruler::QueenElizabethI,
            Civilization::French => Ruler::Napoleon,
            Civilization::German => Ruler::FrederickTheGreat,
            Civilization::Greek => Ruler::Alexander,
            Civilization::Indian => Ruler::Gandhi,
            Civilization::Mongol => Ruler::GenghisKhan,
            Civilization::Roman => Ruler::JuliusCaesar,
            Civilization::Russian => Ruler::Stalin,
            Civilization::Zulu => Ruler::Shaka,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn each_civilization_has_a_ruler() {
        assert_eq!(Civilization::American.ruler(), Ruler::AbrahamLincoln);
        assert_eq!(Civilization::Aztec.ruler(), Ruler::Montezuma);
        assert_eq!(Civilization::Babylonian.ruler(), Ruler::Hammurabi);
        assert_eq!(Civilization::Chinese.ruler(), Ruler::MaoZedong);
        assert_eq!(Civilization::Egyptian.ruler(), Ruler::Ramses);
        assert_eq!(Civilization::English.ruler(), Ruler::QueenElizabethI);
        assert_eq!(Civilization::French.ruler(), Ruler::Napoleon);
        assert_eq!(Civilization::German.ruler(), Ruler::FrederickTheGreat);
        assert_eq!(Civilization::Greek.ruler(), Ruler::Alexander);
        assert_eq!(Civilization::Indian.ruler(), Ruler::Gandhi);
        assert_eq!(Civilization::Mongol.ruler(), Ruler::GenghisKhan);
        assert_eq!(Civilization::Roman.ruler(), Ruler::JuliusCaesar);
        assert_eq!(Civilization::Russian.ruler(), Ruler::Stalin);
        assert_eq!(Civilization::Zulu.ruler(), Ruler::Shaka);
    }

    #[test]
    fn the_civilization_list_is_complete() {
        assert_eq!(Civilization::iter().count(), 14);
        assert!(Civilization::iter().any(|c| c == Civilization::American));
        assert!(Civilization::iter().any(|c| c == Civilization::Zulu));
        assert!(Civilization::iter().any(|c| c == Civilization::Roman));
    }

    #[test]
    fn each_civilization_has_a_display_name() {
        assert_eq!(Civilization::American.display_name(), "American");
        assert_eq!(Civilization::Babylonian.display_name(), "Babylonian");
        assert_eq!(Civilization::Zulu.display_name(), "Zulu");
    }

    #[test]
    fn each_civilization_has_a_motto() {
        for civ in Civilization::iter() {
            assert!(!civ.motto().is_empty());
        }
        assert_eq!(Civilization::Roman.motto(), "Veni, vidi, vici");
        assert_eq!(Civilization::Indian.motto(), "Truth alone triumphs");
    }
}
