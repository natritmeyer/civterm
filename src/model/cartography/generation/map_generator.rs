use crate::model::cartography::{Direction, Location, Map};
use crate::model::geography::Terrain;
use crate::utils::Rng;
use strum::IntoEnumIterator;

pub struct MapGenerator {
    rng: Rng,
}

impl MapGenerator {
    pub fn new(seed: u64) -> Self {
        MapGenerator {
            rng: Rng::new(seed),
        }
    }

    pub fn with_rng(rng: Rng) -> Self {
        MapGenerator { rng }
    }

    pub fn rng(&self) -> &Rng {
        &self.rng
    }

    /// Generate a map by running the full pipeline of terrain methods.
    pub fn generate(&mut self, width: usize, height: usize) -> Map {
        let mut map = Map::new(width, height);
        self.set_to_ocean(&mut map);
        self.set_border_tundra(&mut map);
        self.seed_plains(&mut map, 4);
        self.grow_continents(&mut map, 200);
        self.fill_enclosed_water_with_grass(&mut map);
        map
    }

    /// Convert any water tile that borders land in all but one direction into
    /// grassland, so inland lakes and lagoons left over from continent growth
    /// become walkable land on the final map. Off-map directions count as
    /// neither land nor water, so edge tiles need every on-map neighbour to be
    /// land to qualify.
    pub fn fill_enclosed_water_with_grass(&self, map: &mut Map) {
        let candidates: Vec<Location> = (0..map.height)
            .flat_map(|y| (0..map.width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| map.tile_at(*location).terrain.is_water())
            .collect();
        for location in candidates {
            let land = Direction::iter()
                .filter_map(|direction| map.destination(location, direction))
                .filter(|neighbour| map.tile_at(*neighbour).terrain.is_land())
                .count();
            let water = Direction::iter()
                .filter_map(|direction| map.destination(location, direction))
                .filter(|neighbour| map.tile_at(*neighbour).terrain.is_water())
                .count();
            // All but one direction land: zero or one water neighbour.
            if water <= 1 && land + water >= 2 {
                map.tile_at_mut(location).terrain = Terrain::Grassland;
            }
        }
    }

    /// Set every tile on the map to ocean.
    pub fn set_to_ocean(&self, map: &mut Map) {
        for y in 0..map.height {
            for x in 0..map.width {
                map.tile_at_mut(Location::new(x as u16, y as u16)).terrain = Terrain::Ocean;
            }
        }
    }

    /// Set the top and bottom rows of tiles to tundra.
    pub fn set_border_tundra(&self, map: &mut Map) {
        for x in 0..map.width {
            map.tile_at_mut(Location::new(x as u16, 0)).terrain = Terrain::Tundra;
            map.tile_at_mut(Location::new(x as u16, (map.height - 1) as u16))
                .terrain = Terrain::Tundra;
        }
    }

    /// Drop `count` tiles of plains randomly within the middle 60% of the map.
    pub fn seed_plains(&mut self, map: &mut Map, count: u32) {
        let (band_top, band_bottom) = middle_band(map.height);
        for _ in 0..count {
            let x = self.rng.in_range(map.width as u32) as u16;
            let y = band_top + self.rng.in_range((band_bottom - band_top) as u32) as u16;
            map.tile_at_mut(Location::new(x, y)).terrain = Terrain::Plains;
        }
    }

    /// Grow continents from the seeded plains tiles using a random walk.
    /// Every seeded plains tile starts its own walk of `iterations` steps;
    /// each step drops land onto a random water neighbour within the middle
    /// 60% band, biased toward the neighbour that borders the most already
    /// settled tiles so continents grow chunky rather than stringy. Each walk
    /// is assigned its own region and never settles a tile that touches
    /// another continent, so the lands stay separate. Terrain is mostly
    /// plains/grassland with some hills, mountains, forests and desert,
    /// weighted toward grassland in temperate bands; desert becomes more
    /// likely toward the equator. The walk never spills onto the tundra
    /// borders and never overwrites a tile it has already settled.
    pub fn grow_continents(&mut self, map: &mut Map, iterations: u32) {
        let equator = map.height as f32 * 0.5;
        let seeds: Vec<Location> = (0..map.height)
            .flat_map(|y| (0..map.width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| map.tile_at(*location).terrain == Terrain::Plains)
            .collect();

        let mut regions = vec![vec![None; map.width]; map.height];
        for (region, seed) in seeds.iter().enumerate() {
            regions[seed.y as usize][seed.x as usize] = Some(region);
        }

        for (region, seed) in seeds.iter().enumerate() {
            let mut current = *seed;
            for _ in 0..iterations {
                let Some(next) = self.walk_step(map, &regions, region, current) else {
                    break;
                };
                let terrain = self.random_land(next.y as f32, equator);
                map.tile_at_mut(next).terrain = terrain;
                regions[next.y as usize][next.x as usize] = Some(region);
                current = next;
            }
        }
    }

    /// Pick the next tile for the walk: a random water neighbour that borders
    /// the most settled tiles of this walk's own continent. Every candidate
    /// is weighted by how many same-region neighbours it has, so the walk
    /// prefers filling in next to established land rather than striking out
    /// into open ocean. Candidates touching another continent are rejected so
    /// continents never crash into each other.
    fn walk_step(
        &mut self,
        map: &Map,
        regions: &[Vec<Option<usize>>],
        mine: usize,
        from: Location,
    ) -> Option<Location> {
        let (band_top, band_bottom) = middle_band(map.height);
        let candidates: Vec<(Location, u32)> = Direction::iter()
            .filter_map(|direction| {
                let next = map.destination(from, direction)?;
                if !(band_top..band_bottom).contains(&next.y)
                    || !map.tile_at(next).terrain.is_water()
                    || self.borders_foreign(map, regions, mine, next)
                {
                    return None;
                }
                let borders = self.same_region_neighbours(map, regions, mine, next);
                Some((next, borders))
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        let total: u32 = candidates.iter().map(|&(_, borders)| borders).sum();
        let mut roll = self.rng.in_range(total);
        let mut fallback = candidates[0].0;
        for (next, borders) in candidates {
            if roll < borders {
                return Some(next);
            }
            roll -= borders;
            fallback = next;
        }
        Some(fallback)
    }

    /// Whether `location` sits next to any land belonging to a different
    /// continent. Water and this walk's own land don't count.
    fn borders_foreign(
        &self,
        map: &Map,
        regions: &[Vec<Option<usize>>],
        mine: usize,
        location: Location,
    ) -> bool {
        Direction::iter().any(|direction| {
            map.destination(location, direction)
                .is_some_and(|neighbour| {
                    matches!(
                        regions[neighbour.y as usize][neighbour.x as usize],
                        Some(region) if region != mine
                    )
                })
        })
    }

    /// How many of `location`'s neighbours are settled by this walk's own
    /// continent. Tundra borders don't count.
    fn same_region_neighbours(
        &self,
        map: &Map,
        regions: &[Vec<Option<usize>>],
        mine: usize,
        location: Location,
    ) -> u32 {
        Direction::iter()
            .filter_map(|direction| map.destination(location, direction))
            .filter(|neighbour| regions[neighbour.y as usize][neighbour.x as usize] == Some(mine))
            .count() as u32
    }

    /// Pick a land terrain, favouring plains/grassland. Near the equator
    /// the chance of desert is higher.
    fn random_land(&mut self, latitude: f32, equator: f32) -> Terrain {
        let distance_from_equator = (latitude - equator).abs() / equator.max(1.0);
        let desert_chance = ((1.0 - distance_from_equator) * 20.0).clamp(2.0, 20.0) as u32;

        let roll = self.rng.in_range(100) + 1;
        if roll <= desert_chance {
            return Terrain::Desert;
        }
        if roll <= desert_chance + 10 {
            return Terrain::Mountain;
        }
        if roll <= desert_chance + 25 {
            return Terrain::Hills;
        }
        if roll <= desert_chance + 40 {
            return Terrain::Forest;
        }
        if roll <= desert_chance + 70 {
            return Terrain::Grassland;
        }
        Terrain::Plains
    }
}

/// The middle 60% of the map's rows: everything between 20% and 80% of the
/// height, excluding the tundra border rows. Degenerate maps fall back to the
/// interior rows.
fn middle_band(height: usize) -> (u16, u16) {
    let top = (height as f32 * 0.2).ceil() as u16;
    let bottom = (height as f32 * 0.8).floor() as u16;
    if top < bottom {
        (top.max(1), bottom.min(height as u16 - 1))
    } else if height >= 3 {
        (1, height as u16 - 1)
    } else {
        (0, height as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_to_ocean_floods_every_tile() {
        let generator = MapGenerator::new(1);
        let mut map = Map::new(4, 4);
        map.tile_at_mut(Location::new(1, 1)).terrain = Terrain::Plains;
        generator.set_to_ocean(&mut map);
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(map.tile_at(Location::new(x, y)).terrain, Terrain::Ocean);
            }
        }
    }

    #[test]
    fn set_border_tundra_marks_only_the_top_and_bottom_rows() {
        let generator = MapGenerator::new(1);
        let mut map = Map::new(4, 4);
        generator.set_to_ocean(&mut map);
        generator.set_border_tundra(&mut map);
        for x in 0..4 {
            assert_eq!(map.tile_at(Location::new(x, 0)).terrain, Terrain::Tundra);
            assert_eq!(map.tile_at(Location::new(x, 3)).terrain, Terrain::Tundra);
        }
        for y in 1..3 {
            for x in 0..4 {
                assert_eq!(map.tile_at(Location::new(x, y)).terrain, Terrain::Ocean);
            }
        }
    }

    #[test]
    fn seed_plains_drops_the_requested_count_in_the_middle_band() {
        let mut generator = MapGenerator::new(1);
        let mut map = Map::new(100, 100);
        generator.set_to_ocean(&mut map);
        generator.seed_plains(&mut map, 4);
        let count = (0..100)
            .flat_map(|y| (0..100).map(move |x| Location::new(x, y)))
            .filter(|location| map.tile_at(*location).terrain == Terrain::Plains)
            .count();
        assert_eq!(count, 4);
        let (band_top, band_bottom) = middle_band(100);
        for y in 0..100 {
            for x in 0..100 {
                let location = Location::new(x, y);
                if map.tile_at(location).terrain == Terrain::Plains {
                    assert!(
                        (band_top..band_bottom).contains(&y),
                        "seed outside the middle band"
                    );
                }
            }
        }
    }

    #[test]
    fn grow_continents_adds_land_tiles() {
        let mut generator = MapGenerator::new(1);
        let mut map = Map::new(40, 40);
        generator.set_to_ocean(&mut map);
        generator.seed_plains(&mut map, 4);
        let before = land_count(&map);
        generator.grow_continents(&mut map, 500);
        assert!(land_count(&map) > before);
    }

    #[test]
    fn the_walk_grows_from_every_seed() {
        let mut generator = MapGenerator::new(1);
        let mut map = Map::new(40, 40);
        generator.set_to_ocean(&mut map);
        generator.seed_plains(&mut map, 4);
        let seeds: Vec<Location> = (0..map.height)
            .flat_map(|y| (0..map.width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| map.tile_at(*location).terrain == Terrain::Plains)
            .collect();
        assert_eq!(seeds.len(), 4, "expected four plains seeds");
        generator.grow_continents(&mut map, 10);
        for seed in seeds {
            let grew = Direction::iter().any(|direction| {
                map.destination(seed, direction)
                    .is_some_and(|neighbour| map.tile_at(neighbour).terrain.is_land())
            });
            assert!(grew, "no land grew from seed {seed:?}");
        }
    }

    #[test]
    fn grow_continents_keeps_each_continent_separate() {
        let mut generator = MapGenerator::new(1);
        let mut map = Map::new(40, 40);
        generator.set_to_ocean(&mut map);
        for seed in [Location::new(8, 20), Location::new(32, 20)] {
            map.tile_at_mut(seed).terrain = Terrain::Plains;
        }
        generator.grow_continents(&mut map, 5_000);
        assert_eq!(
            land_components(&map),
            2,
            "continents crashed into each other"
        );
    }

    fn land_components(map: &Map) -> usize {
        let mut visited = vec![vec![false; map.width]; map.height];
        let mut components = 0;
        for y in 0..map.height {
            for x in 0..map.width {
                let start = Location::new(x as u16, y as u16);
                if map.tile_at(start).terrain.is_land() && !visited[y][x] {
                    components += 1;
                    visited[y][x] = true;
                    let mut stack = vec![start];
                    while let Some(location) = stack.pop() {
                        for direction in Direction::iter() {
                            if let Some(neighbour) = map.destination(location, direction) {
                                let ny = neighbour.y as usize;
                                let nx = neighbour.x as usize;
                                if map.tile_at(neighbour).terrain.is_land() && !visited[ny][nx] {
                                    visited[ny][nx] = true;
                                    stack.push(neighbour);
                                }
                            }
                        }
                    }
                }
            }
        }
        components
    }

    #[test]
    fn grown_land_stays_within_the_middle_band() {
        let mut generator = MapGenerator::new(1);
        let mut map = Map::new(40, 40);
        generator.set_to_ocean(&mut map);
        generator.seed_plains(&mut map, 4);
        generator.grow_continents(&mut map, 10_000);
        let (band_top, band_bottom) = middle_band(40);
        for y in 0..40 {
            for x in 0..40 {
                let location = Location::new(x, y);
                if map.tile_at(location).terrain.is_land() {
                    assert!(
                        (band_top..band_bottom).contains(&y),
                        "grown land escaped the middle band"
                    );
                }
            }
        }
    }

    #[test]
    fn the_walk_never_overwrites_land_tiles() {
        let mut generator = MapGenerator::new(1);
        let mut map = Map::new(40, 40);
        generator.set_to_ocean(&mut map);
        generator.seed_plains(&mut map, 4);
        generator.grow_continents(&mut map, 500);
        let before: Vec<(Location, Terrain)> = (0..map.height)
            .flat_map(|y| (0..map.width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| map.tile_at(*location).terrain.is_land())
            .map(|location| (location, map.tile_at(location).terrain))
            .collect();
        generator.grow_continents(&mut map, 10_000);
        for (location, terrain) in before {
            assert_eq!(
                map.tile_at(location).terrain,
                terrain,
                "a land tile was overwritten by the walk"
            );
        }
    }

    #[test]
    fn tundra_rows_remain_and_are_not_adjacent_to_land() {
        let mut generator = MapGenerator::new(1);
        let map = generator.generate(40, 30);
        let last = map.height - 1;
        for x in 0..map.width {
            let location = Location::new(x as u16, 0);
            assert_eq!(map.tile_at(location).terrain, Terrain::Tundra);
            let location = Location::new(x as u16, last as u16);
            assert_eq!(map.tile_at(location).terrain, Terrain::Tundra);
            assert!(
                !map.tile_at(Location::new(x as u16, 1)).terrain.is_land(),
                "land adjacent to the top tundra row"
            );
            assert!(
                !map.tile_at(Location::new(x as u16, (last - 1) as u16))
                    .terrain
                    .is_land(),
                "land adjacent to the bottom tundra row"
            );
        }
    }

    #[test]
    fn generate_runs_the_full_pipeline_and_draws_the_final_map() {
        let mut generator = MapGenerator::new(1);
        let map = generator.generate(80, 50);
        assert!(land_count(&map) > 0);
    }

    #[test]
    fn enclosed_water_tiles_become_grassland() {
        let generator = MapGenerator::new(1);
        let mut map = Map::new(5, 5);
        generator.set_to_ocean(&mut map);
        // Fully ring the centre tile (2,2): zero water neighbours.
        for y in 0..5 {
            for x in 0..5 {
                if x != 2 || y != 2 {
                    map.tile_at_mut(Location::new(x as u16, y as u16)).terrain = Terrain::Grassland;
                }
            }
        }
        generator.fill_enclosed_water_with_grass(&mut map);
        assert_eq!(
            map.tile_at(Location::new(2, 2)).terrain,
            Terrain::Grassland,
            "a fully enclosed water tile should fill in"
        );
    }

    #[test]
    fn a_one_empty_neighbour_water_tile_is_filled() {
        let generator = MapGenerator::new(1);
        let mut map = Map::new(5, 5);
        generator.set_to_ocean(&mut map);
        // Centre (2,2): make seven of its eight neighbours land, leaving one
        // open water neighbour so the centre has all-but-one land borders.
        for (x, y) in [(1, 1), (3, 1), (1, 2), (3, 2), (1, 3), (2, 3), (3, 3)] {
            map.tile_at_mut(Location::new(x, y)).terrain = Terrain::Grassland;
        }
        generator.fill_enclosed_water_with_grass(&mut map);
        assert_eq!(
            map.tile_at(Location::new(2, 2)).terrain,
            Terrain::Grassland,
            "a tile with all but one land neighbour should fill in"
        );
    }

    #[test]
    fn open_water_stays_water() {
        let generator = MapGenerator::new(1);
        let mut map = Map::new(4, 4);
        generator.set_to_ocean(&mut map);
        // (1,1) has several water neighbours (open ocean), so it must not be
        // converted even though it also touches some land.
        map.tile_at_mut(Location::new(0, 0)).terrain = Terrain::Grassland;
        map.tile_at_mut(Location::new(0, 1)).terrain = Terrain::Grassland;
        generator.fill_enclosed_water_with_grass(&mut map);
        assert_eq!(
            map.tile_at(Location::new(1, 1)).terrain,
            Terrain::Ocean,
            "open water should remain ocean"
        );
    }

    fn land_count(map: &Map) -> usize {
        (0..map.height)
            .flat_map(|y| (0..map.width).map(move |x| Location::new(x as u16, y as u16)))
            .filter(|location| map.tile_at(*location).terrain.is_land())
            .count()
    }
}
