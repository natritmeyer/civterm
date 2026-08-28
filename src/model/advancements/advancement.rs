#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advancement {
    Alphabet,
    Astronomy,
    Banking,
    BridgeBuilding,
    BronzeWorking,
    CeremonialBurial,
    Chemistry,
    Chivalry,
    CodeOfLaws,
    Construction,
    Currency,
    Engineering,
    Feudalism,
    HorsebackRiding,
    Invention,
    IronWorking,
    Literacy,
    Magnetism,
    MapMaking,
    Masonry,
    Mathematics,
    Medicine,
    Mysticism,
    Navigation,
    Philosophy,
    Physics,
    Pottery,
    Recycling,
    Refining,
    Religion,
    TheoryOfGravity,
    Trade,
    University,
    Wheel,
    Writing,
}

impl Advancement {
    pub fn prerequisites(&self) -> &'static [Advancement] {
        match self {
            Advancement::Alphabet => &[],
            Advancement::Astronomy => &[Advancement::Mysticism, Advancement::Mathematics],
            Advancement::Banking => &[Advancement::Trade],
            Advancement::BridgeBuilding => &[Advancement::IronWorking, Advancement::Alphabet],
            Advancement::BronzeWorking => &[],
            Advancement::CeremonialBurial => &[],
            Advancement::Chemistry => &[Advancement::University, Advancement::Medicine],
            Advancement::Chivalry => &[Advancement::Feudalism, Advancement::HorsebackRiding],
            Advancement::CodeOfLaws => &[Advancement::Alphabet],
            Advancement::Construction => &[Advancement::Masonry, Advancement::Currency],
            Advancement::Currency => &[Advancement::BronzeWorking],
            Advancement::Engineering => &[Advancement::Wheel, Advancement::Construction],
            Advancement::Feudalism => &[Advancement::Masonry],
            Advancement::HorsebackRiding => &[],
            Advancement::Invention => &[Advancement::Engineering, Advancement::Literacy],
            Advancement::IronWorking => &[Advancement::BronzeWorking],
            Advancement::Literacy => &[Advancement::Writing, Advancement::CodeOfLaws],
            Advancement::Magnetism => &[Advancement::Navigation, Advancement::Physics],
            Advancement::MapMaking => &[Advancement::Alphabet],
            Advancement::Masonry => &[],
            Advancement::Mathematics => &[Advancement::Alphabet, Advancement::Masonry],
            Advancement::Medicine => &[Advancement::Philosophy, Advancement::Trade],
            Advancement::Mysticism => &[Advancement::CeremonialBurial],
            Advancement::Navigation => &[Advancement::MapMaking, Advancement::Astronomy],
            Advancement::Philosophy => &[Advancement::Mysticism, Advancement::Literacy],
            Advancement::Physics => &[Advancement::Mathematics, Advancement::Navigation],
            Advancement::Pottery => &[],
            Advancement::Recycling => &[],
            Advancement::Refining => &[Advancement::Chemistry],
            Advancement::Religion => &[Advancement::Philosophy, Advancement::Writing],
            Advancement::TheoryOfGravity => &[Advancement::Astronomy, Advancement::University],
            Advancement::Trade => &[Advancement::Currency, Advancement::CodeOfLaws],
            Advancement::University => &[Advancement::Mathematics, Advancement::Philosophy],
            Advancement::Wheel => &[],
            Advancement::Writing => &[Advancement::Alphabet],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn founding_advances_have_no_prerequisites() {
        assert!(Advancement::Alphabet.prerequisites().is_empty());
        assert!(Advancement::BronzeWorking.prerequisites().is_empty());
        assert!(Advancement::CeremonialBurial.prerequisites().is_empty());
        assert!(Advancement::HorsebackRiding.prerequisites().is_empty());
        assert!(Advancement::Masonry.prerequisites().is_empty());
        assert!(Advancement::Pottery.prerequisites().is_empty());
        assert!(Advancement::Wheel.prerequisites().is_empty());
    }

    #[test]
    fn single_prerequisite_advances() {
        assert_eq!(
            Advancement::CodeOfLaws.prerequisites(),
            &[Advancement::Alphabet]
        );
        assert_eq!(
            Advancement::MapMaking.prerequisites(),
            &[Advancement::Alphabet]
        );
        assert_eq!(
            Advancement::Mysticism.prerequisites(),
            &[Advancement::CeremonialBurial]
        );
        assert_eq!(
            Advancement::Writing.prerequisites(),
            &[Advancement::Alphabet]
        );
    }

    #[test]
    fn two_prerequisite_advances() {
        assert_eq!(
            Advancement::Astronomy.prerequisites(),
            &[Advancement::Mysticism, Advancement::Mathematics]
        );
        assert_eq!(
            Advancement::Chivalry.prerequisites(),
            &[Advancement::Feudalism, Advancement::HorsebackRiding]
        );
        assert_eq!(
            Advancement::Engineering.prerequisites(),
            &[Advancement::Wheel, Advancement::Construction]
        );
        assert_eq!(
            Advancement::Trade.prerequisites(),
            &[Advancement::Currency, Advancement::CodeOfLaws]
        );
        assert_eq!(
            Advancement::University.prerequisites(),
            &[Advancement::Mathematics, Advancement::Philosophy]
        );
    }
}
