#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Civilization {
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
}
