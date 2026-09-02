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
