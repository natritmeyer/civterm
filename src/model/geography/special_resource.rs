use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
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
    /// The single-cell glyph painted in a tile's spare column, beside the
    /// terrain letter: `♠` coal, `>` fish, `❧` game, `♦` gems, `¤` gold,
    /// `♞` horses, `○` oasis, `Ψ` oil. Each one is one character so it cannot
    /// spill into the neighbouring tile.
    pub fn as_char(&self) -> char {
        match self {
            SpecialResource::Coal => '©', //'♠',
            SpecialResource::Fish => '>',
            SpecialResource::Game => '»', //'❧',
            SpecialResource::Gems => 'ô', //'♦',
            SpecialResource::Gold => '¤',
            SpecialResource::Horses => 'ž', //'♞',
            SpecialResource::Oasis => '@',  //'○'
            SpecialResource::Oil => 'ÿ',    //'Ψ',
        }
    }

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

    /// Gold minted each turn by mining the resource. Only the Gold resource on
    /// a mountain yields any; plain mountains strike no coin.
    pub fn yields_gold(&self) -> u8 {
        match self {
            SpecialResource::Gold => 2,
            SpecialResource::Coal
            | SpecialResource::Fish
            | SpecialResource::Game
            | SpecialResource::Gems
            | SpecialResource::Horses
            | SpecialResource::Oasis
            | SpecialResource::Oil => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_special_resource_has_a_distinct_glyph() {
        let resources = [
            SpecialResource::Coal,
            SpecialResource::Fish,
            SpecialResource::Game,
            SpecialResource::Gems,
            SpecialResource::Gold,
            SpecialResource::Horses,
            SpecialResource::Oasis,
            SpecialResource::Oil,
        ];
        let glyphs: Vec<char> = resources.iter().map(SpecialResource::as_char).collect();
        let mut unique = glyphs.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            unique.len(),
            resources.len(),
            "every resource needs its own glyph"
        );
    }

    #[test]
    fn resource_glyphs_do_not_collide_with_terrain_letters() {
        use crate::model::geography::Terrain;

        let terrain = [
            Terrain::Ocean,
            Terrain::Grassland,
            Terrain::Plains,
            Terrain::Forest,
            Terrain::Hills,
            Terrain::Mountain,
            Terrain::Desert,
            Terrain::Tundra,
            Terrain::Swamp,
            Terrain::Jungle,
        ]
        .map(|terrain| terrain.as_char());
        for resource in [
            SpecialResource::Coal,
            SpecialResource::Fish,
            SpecialResource::Game,
            SpecialResource::Gems,
            SpecialResource::Gold,
            SpecialResource::Horses,
            SpecialResource::Oasis,
            SpecialResource::Oil,
        ] {
            let glyph = resource.as_char();
            assert!(!glyph.is_whitespace() && !glyph.is_control());
            assert!(
                !terrain.contains(&glyph),
                "{glyph} must not be mistaken for a terrain letter"
            );
        }
    }
}
