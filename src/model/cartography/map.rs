use crate::model::cartography::Tile;
use crate::model::cartography::{Direction, Location};
use crate::model::geography::Geography;

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

    pub fn destination(&self, from: Location, direction: Direction) -> Option<Location> {
        let (width, height) = (self.width as isize, self.height as isize);
        let (dx, dy) = direction.delta();
        let x = (from.x as isize + dx).rem_euclid(width);
        let y = from.y as isize + dy;
        if y >= 0 && y < height {
            Some(Location::new(x as u16, y as u16))
        } else {
            None
        }
    }

    /// Render the map as an ASCII grid, one line per row.
    pub fn render_ascii(&self) -> String {
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for row in &self.tiles {
            for tile in row {
                out.push(tile.geography.as_char());
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geography::SpecialResource;

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
        mountain.place_resource(SpecialResource::Gold).unwrap();
        *map.tile_at_mut(Location::new(2, 1)) = mountain.clone();

        assert_eq!(map.tile_at(Location::new(0, 0)).geography, Geography::Ocean);
        assert_eq!(map.tile_at(Location::new(2, 1)), &mountain);
    }

    #[test]
    fn destination_wraps_around_the_east_and_west_edges() {
        let map = Map::new(3, 2);
        assert_eq!(
            map.destination(Location::new(2, 1), Direction::E),
            Some(Location::new(0, 1))
        );
        assert_eq!(
            map.destination(Location::new(0, 1), Direction::W),
            Some(Location::new(2, 1))
        );
    }

    #[test]
    fn destination_is_absent_off_the_north_and_south_edges() {
        let map = Map::new(3, 2);
        assert_eq!(map.destination(Location::new(1, 0), Direction::N), None);
        assert_eq!(map.destination(Location::new(1, 1), Direction::S), None);
    }

    #[test]
    fn render_ascii_prints_one_line_per_row() {
        let mut map = Map::new(3, 2);
        map.tile_at_mut(Location::new(1, 0)).geography = Geography::Grassland;
        let rendered = map.render_ascii();
        assert_eq!(rendered, "~v~\n~~~\n");
    }
}
