use crate::model::geography::Geography;

pub mod location;
pub mod tile;

pub use location::Location;
pub use tile::Tile;

pub struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            tiles: vec![vec![Tile::new(Geography::Ocean); width]; height],
        }
    }

    pub fn tile_at(&self, location: Location) -> &Tile {
        &self.tiles[location.y as usize][location.x as usize]
    }

    pub fn tile_at_mut(&mut self, location: Location) -> &mut Tile {
        &mut self.tiles[location.y as usize][location.x as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_defaults_to_ocean() {
        let map = Map::new(4, 3);
        for y in 0..3 {
            for x in 0..4 {
                assert_eq!(map.tile_at(Location::new(x, y)).geography, Geography::Ocean);
            }
        }
    }

    #[test]
    fn tile_at_returns_the_tile_at_that_location() {
        let mut map = Map::new(3, 2);
        let mut mountain = Tile::new(Geography::Mountain);
        mountain
            .place_resource(crate::model::geography::SpecialResource::Gold)
            .unwrap();
        *map.tile_at_mut(Location::new(2, 1)) = mountain.clone();

        assert_eq!(map.tile_at(Location::new(0, 0)).geography, Geography::Ocean);
        assert_eq!(map.tile_at(Location::new(2, 1)), &mountain);
    }
}
