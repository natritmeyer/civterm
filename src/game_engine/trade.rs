use serde::{Deserialize, Serialize};

use crate::game_engine::game::Game;
use crate::game_engine::{Engine, Event};
use crate::model::cartography::{Location, Map};
use crate::model::cities::{City, CityId};
use crate::model::civilizations::PlayerId;
use crate::model::geography::SpecialResource;
use crate::model::units::{UnitClass, UnitId};

/// How many trade routes a single city can ever be an endpoint of. A city can
/// send three caravans and receive three more before its market is full.
pub const MAX_TRADE_ROUTES: usize = 3;

/// The flat gold a route pays before distance, size and resources are counted.
const ROUTE_BASE_GOLD: u32 = 1;
/// Distance is measured in tiles walked on the wrapped map, so the divisor
/// keeps a single cross-map haul from out-earning a whole empire's tax base.
const ROUTE_DISTANCE_DIVISOR: u32 = 4;
/// Gold each distinct special resource worked by an endpoint adds. A route
/// between two resource towns is worth more than one between two plains.
const ROUTE_RESOURCE_BONUS: u32 = 2;

/// A lasting trade link between two cities, opened when a Caravan walks into a
/// city. It pays `gold` to whichever owner holds each endpoint, once a turn,
/// and lasts until one of the cities is destroyed — a city changing hands does
/// not break the route, it just changes who is paid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeRoute {
    pub from: CityId,
    pub to: CityId,
    pub gold: u32,
}

impl TradeRoute {
    /// Whether `city` is one of the route's two endpoints.
    pub fn touches(&self, city: CityId) -> bool {
        self.from == city || self.to == city
    }

    /// The other end of the route from `city`, or `None` if it does not touch
    /// `city` at all.
    pub fn other_end(&self, city: CityId) -> Option<CityId> {
        if self.from == city {
            Some(self.to)
        } else if self.to == city {
            Some(self.from)
        } else {
            None
        }
    }
}

impl Game {
    /// Every route with `city` as an endpoint.
    pub fn trade_routes_for(&self, city: CityId) -> Vec<&TradeRoute> {
        self.trade_routes
            .iter()
            .filter(|route| route.touches(city))
            .collect()
    }

    /// How many routes end at `city`.
    pub fn trade_route_count(&self, city: CityId) -> usize {
        self.trade_routes
            .iter()
            .filter(|route| route.touches(city))
            .count()
    }

    /// The gold `owner` collects from trade routes in one turn. Each endpoint
    /// pays its own gold to whoever holds it, so a link between two of the
    /// owner's own cities pays that owner twice — a domestic route is worth
    /// more to one power than a foreign one.
    pub fn trade_income(&self, owner: PlayerId) -> u32 {
        self.trade_routes
            .iter()
            .map(|route| {
                let mut gold = 0;
                for end in [route.from, route.to] {
                    if self
                        .cities
                        .iter()
                        .any(|city| city.id() == end && city.owner() == owner)
                    {
                        gold += route.gold;
                    }
                }
                gold
            })
            .sum()
    }

    /// What a route between `from` and `to` would pay at each endpoint today,
    /// or `None` if either city is missing. The value is measured once, when
    /// the caravan arrives, and never recomputed: a route pays what it was
    /// worth the day it opened.
    pub fn trade_route_value(&self, from: CityId, to: CityId) -> Option<u32> {
        let from_city = self.cities.iter().find(|city| city.id() == from)?;
        let to_city = self.cities.iter().find(|city| city.id() == to)?;
        let distance = self.wrapped_distance(from_city.location, to_city.location);
        let mut resources = distinct_worked_resources(from_city, &self.map);
        for resource in distinct_worked_resources(to_city, &self.map) {
            if !resources.contains(&resource) {
                resources.push(resource);
            }
        }
        let distinct = resources.len() as u32;
        Some(
            ROUTE_BASE_GOLD
                + distance / ROUTE_DISTANCE_DIVISOR
                + to_city.population()
                + ROUTE_RESOURCE_BONUS * distinct,
        )
    }

    /// Drop every route that ends at `city`, called when the city is destroyed
    /// so no route outlives an endpoint that no longer exists.
    pub fn remove_trade_routes_touching(&mut self, city: CityId) {
        self.trade_routes.retain(|route| !route.touches(city));
    }

    /// The distance a caravan would walk between two tiles, east/west wrapping
    /// exactly as movement does: the short way round the map, measured as a
    /// chebyshev step count.
    fn wrapped_distance(&self, a: Location, b: Location) -> u32 {
        let width = self.map.width as u32;
        let raw_dx = a.x.abs_diff(b.x) as u32;
        let dx = raw_dx.min(width.saturating_sub(raw_dx));
        let dy = a.y.abs_diff(b.y) as u32;
        dx.max(dy)
    }
}

/// The distinct special resources a city currently works.
fn distinct_worked_resources(city: &City, map: &Map) -> Vec<SpecialResource> {
    let mut found: Vec<SpecialResource> = Vec::new();
    for location in city.worked_tiles() {
        if let Some(resource) = map.tile_at(*location).resource()
            && !found.contains(&resource)
        {
            found.push(resource);
        }
    }
    found
}

impl Engine {
    /// A Caravan arriving on a city tile turns the journey into a lasting trade
    /// route: it is consumed and never reappears on the board. Returns true
    /// when the caravan was spent, so the caller can skip the ordinary move
    /// announcement. The same peace-and-room rules the player's route obeys
    /// apply to a rival's caravan — this is the one path that opens a route.
    pub(super) fn try_establish_trade_route(
        &mut self,
        unit: UnitId,
        destination: Location,
    ) -> bool {
        let Some(caravan) = self.game.units.iter().find(|u| u.id() == unit) else {
            return false;
        };
        if caravan.unit_class != UnitClass::Caravan {
            return false;
        }
        let owner = caravan.owner();
        let Some(home) = caravan.home_city() else {
            return false;
        };
        let Some(partner_city) = self
            .game
            .cities
            .iter()
            .find(|city| city.location == destination)
        else {
            return false;
        };
        let partner = partner_city.id();
        let partner_owner = partner_city.owner();
        let partner_name = partner_city.name.clone();
        // A caravan standing in its own home city has gone nowhere.
        if partner == home {
            return false;
        }
        // A route is a peaceful link: at war the destination's garrison meets
        // the caravan as an attacker, not as a trade partner.
        if self.game.at_war(owner, partner_owner) {
            return false;
        }
        if self.game.trade_route_count(home) >= MAX_TRADE_ROUTES
            || self.game.trade_route_count(partner) >= MAX_TRADE_ROUTES
        {
            self.events.push(Event::for_player(
                owner,
                format!("No room for another trade route with {partner_name}"),
            ));
            return false;
        }
        let Some(gold) = self.game.trade_route_value(home, partner) else {
            return false;
        };
        let home_name = self
            .game
            .cities
            .iter()
            .find(|city| city.id() == home)
            .map(|city| city.name.clone())
            .unwrap_or_default();
        self.game.trade_routes.push(TradeRoute {
            from: home,
            to: partner,
            gold,
        });
        self.game.remove_unit(unit);
        self.events.push(Event::between(
            owner,
            partner_owner,
            format!("{home_name} opens a trade route with {partner_name}"),
        ));
        true
    }
}
