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

    /// City names in the order they are awarded as the civilization founds
    /// new settlements. The capital name always comes first.
    pub fn city_names(&self) -> &'static [&'static str] {
        match self {
            Civilization::American => &[
                "Washington",
                "Boston",
                "New York",
                "Philadelphia",
                "Atlanta",
                "Chicago",
                "San Francisco",
                "Los Angeles",
                "Houston",
                "New Orleans",
                "Detroit",
                "Dallas",
                "Denver",
                "Seattle",
                "Miami",
                "Portland",
                "St. Louis",
                "Cincinnati",
                "Pittsburgh",
                "Richmond",
            ],
            Civilization::Aztec => &[
                "Tenochtitlan",
                "Texcoco",
                "Tlacopan",
                "Cholula",
                "Tlatelolco",
                "Xochimilco",
                "Chalco",
                "Tlaxcala",
                "Teotihuacan",
                "Acolman",
                "Tula",
                "Atzcapotzalco",
                "Coatlan",
                "Iztapalapa",
                "Oaxaca",
                "Malinalco",
                "Cuernavaca",
                "Xalapa",
                "Capilco",
                "Vera Cruz",
            ],
            Civilization::Babylonian => &[
                "Babylon",
                "Nineveh",
                "Ashur",
                "Ur",
                "Akkad",
                "Eridu",
                "Kish",
                "Lagash",
                "Nippur",
                "Larsa",
                "Uruk",
                "Umma",
                "Isin",
                "Marad",
                "Shuruppak",
                "Borsippa",
                "Sippar",
                "Der",
                "Opis",
                "Nuzi",
            ],
            Civilization::Chinese => &[
                "Beijing",
                "Shanghai",
                "Nanjing",
                "Xian",
                "Guangzhou",
                "Chengdu",
                "Wuhan",
                "Hong Kong",
                "Taipei",
                "Lanzhou",
                "Tientsin",
                "Macao",
                "Amoy",
                "Fuzhou",
                "Luoyang",
                "Hangzhou",
                "Kaifeng",
                "Harbin",
                "Jinan",
                "Chengde",
            ],
            Civilization::Egyptian => &[
                "Thebes",
                "Memphis",
                "Heliopolis",
                "Alexandria",
                "Giza",
                "Elephantine",
                "Abydos",
                "Buto",
                "Sais",
                "Bubastis",
                "Karnak",
                "Luxor",
                "Aswan",
                "Dendera",
                "Edfu",
                "Esna",
                "Kom Ombo",
                "Abu Simbel",
                "Cairo",
                "Pi-Ramesses",
            ],
            Civilization::English => &[
                "London",
                "York",
                "Manchester",
                "Liverpool",
                "Birmingham",
                "Oxford",
                "Cambridge",
                "Bristol",
                "Leeds",
                "Sheffield",
                "Newcastle",
                "Nottingham",
                "Plymouth",
                "Exeter",
                "Canterbury",
                "Norwich",
                "Southampton",
                "Portsmouth",
                "Bath",
                "Dover",
            ],
            Civilization::French => &[
                "Paris",
                "Orleans",
                "Lyons",
                "Bordeaux",
                "Tours",
                "Rheims",
                "Marseilles",
                "Chartres",
                "Grenoble",
                "Nice",
                "Avignon",
                "Rouen",
                "Toulouse",
                "Dijon",
                "Amiens",
                "Nantes",
                "Brest",
                "Strasbourg",
                "Calais",
                "Versailles",
            ],
            Civilization::German => &[
                "Berlin",
                "Hamburg",
                "Munich",
                "Cologne",
                "Frankfurt",
                "Stuttgart",
                "Leipzig",
                "Dresden",
                "Nuremberg",
                "Bremen",
                "Bonn",
                "Karlsruhe",
                "Freiburg",
                "Mainz",
                "Trier",
                "Augsburg",
                "Hannover",
                "Kiel",
                "Essen",
                "Dortmund",
            ],
            Civilization::Greek => &[
                "Athens",
                "Sparta",
                "Corinth",
                "Knossos",
                "Argos",
                "Delphi",
                "Mycenae",
                "Pylos",
                "Pharsalos",
                "Thermopylae",
                "Megara",
                "Marathon",
                "Olympia",
                "Ephesus",
                "Halicarnassus",
                "Miletus",
                "Rhodes",
                "Salamis",
                "Patras",
                "Iolcos",
            ],
            Civilization::Indian => &[
                "Delhi",
                "Bombay",
                "Madras",
                "Calcutta",
                "Lahore",
                "Bangalore",
                "Hyderabad",
                "Jaipur",
                "Agra",
                "Varanasi",
                "Kanpur",
                "Amritsar",
                "Srinagar",
                "Karachi",
                "Pune",
                "Ahmedabad",
                "Indore",
                "Lucknow",
                "Patna",
                "Chandigarh",
            ],
            Civilization::Mongol => &[
                "Karakorum",
                "Samarkand",
                "Bukhara",
                "Tabriz",
                "Kazan",
                "Astrakhan",
                "Ulaanbaatar",
                "Kharkhorin",
                "Erdenet",
                "Darkhan",
                "Altai",
                "Bulgan",
                "Khovd",
                "Dalandzadgad",
                "Saikhan",
                "Choibalsan",
                "Murun",
                "Olgii",
                "Uliastai",
                "Tsetserleg",
            ],
            Civilization::Roman => &[
                "Rome",
                "Veii",
                "Antium",
                "Cumae",
                "Neapolis",
                "Ravenna",
                "Syracuse",
                "Arretium",
                "Mediolanum",
                "Pisae",
                "Tarentum",
                "Brundisium",
                "Salernum",
                "Pompeii",
                "Capua",
                "Ostia",
                "Praeneste",
                "Aquileia",
                "Genua",
                "Patavium",
            ],
            Civilization::Russian => &[
                "Moscow",
                "Leningrad",
                "Novgorod",
                "Kiev",
                "Minsk",
                "Smolensk",
                "Odessa",
                "Vladivostok",
                "Stalingrad",
                "Kursk",
                "Bryansk",
                "Yaroslavl",
                "Tula",
                "Rostov",
                "Pskov",
                "Tver",
                "Murmansk",
                "Saratov",
                "Voronezh",
                "Perm",
            ],
            Civilization::Zulu => &[
                "Zimbabwe",
                "Ulundi",
                "Bapedi",
                "Isandhlwana",
                "Ndondakusuka",
                "Intombe",
                "Umfolozi",
                "Hlobane",
                "Ongoye",
                "Nongoma",
                "Eshowe",
                "Mahlabatini",
                "Ceza",
                "Mkuze",
                "Phongolo",
                "Tugela",
                "Nguthu",
                "Kwamagwaza",
                "Hlangana",
                "Nobamba",
            ],
        }
    }

    pub fn capital_name(&self) -> &'static str {
        self.city_names()[0]
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
    fn each_civilization_has_a_capital_name() {
        for civ in Civilization::iter() {
            assert!(!civ.capital_name().is_empty());
        }
        assert_eq!(Civilization::English.capital_name(), "London");
        assert_eq!(Civilization::Roman.capital_name(), "Rome");
        assert_eq!(Civilization::Aztec.capital_name(), "Tenochtitlan");
    }

    #[test]
    fn each_civilization_has_an_ordered_list_of_city_names() {
        for civ in Civilization::iter() {
            let names = civ.city_names();
            assert!(names.len() >= 20, "{civ:?} has only {} names", names.len());
            assert_eq!(
                names[0],
                civ.capital_name(),
                "{civ:?} capital must come first"
            );
            assert!(names.iter().all(|n| !n.is_empty()));
            let mut sorted = names.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), names.len(), "{civ:?} repeats a city name");
        }
        assert_eq!(Civilization::English.city_names()[1], "York");
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
