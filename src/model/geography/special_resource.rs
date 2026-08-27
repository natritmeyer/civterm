#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialResource {
    Coal,
    Fish,
    Game,
    Gems,
    Gold,
    Horses,
    Oasis,
    Oil,
}

impl SpecialResource {
    pub fn yields_food(&self) -> u8 {
        match self {
            SpecialResource::Fish | SpecialResource::Game => 1,
            SpecialResource::Oasis => 3,
            SpecialResource::Coal
            | SpecialResource::Gems
            | SpecialResource::Gold
            | SpecialResource::Horses
            | SpecialResource::Oil => 0,
        }
    }

    pub fn yields_resources(&self) -> u8 {
        match self {
            SpecialResource::Coal | SpecialResource::Horses => 1,
            SpecialResource::Oil => 2,
            SpecialResource::Fish
            | SpecialResource::Game
            | SpecialResource::Gems
            | SpecialResource::Gold
            | SpecialResource::Oasis => 0,
        }
    }

    pub fn yields_trade(&self) -> u8 {
        match self {
            SpecialResource::Gems | SpecialResource::Gold => 2,
            SpecialResource::Coal
            | SpecialResource::Fish
            | SpecialResource::Game
            | SpecialResource::Horses
            | SpecialResource::Oasis
            | SpecialResource::Oil => 0,
        }
    }
}
