use super::*;
use crate::model::cartography::Tile;
use crate::model::cities::CityId;
use crate::model::civilizations::Civilization;
use crate::model::geography::Terrain;
use crate::model::units::{UnitClass, UnitId, UnitOrder};
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier, Style};

fn terrain_colors(terrain: Terrain) -> (Color, Color) {
    use Terrain::*;
    match terrain {
        Ocean => (Color::Rgb(40, 90, 160), Color::Rgb(20, 40, 80)),
        Plains => (Color::Rgb(185, 205, 110), Color::Rgb(80, 95, 45)),
        Jungle => (Color::Rgb(140, 210, 90), Color::Rgb(45, 95, 40)),
        Forest => (Color::Rgb(60, 150, 60), Color::Rgb(25, 70, 30)),
        Hills => (Color::Rgb(150, 130, 90), Color::Rgb(70, 60, 40)),
        Mountain => (Color::Rgb(170, 170, 180), Color::Rgb(80, 80, 90)),
        Desert => (Color::Rgb(210, 200, 130), Color::Rgb(110, 100, 50)),
        Tundra => (Color::Rgb(170, 200, 200), Color::Rgb(70, 90, 90)),
        Swamp => (Color::Rgb(90, 150, 120), Color::Rgb(40, 80, 60)),
        Grassland => (Color::Rgb(60, 160, 80), Color::Rgb(25, 75, 40)),
    }
}

/// The watermark painted in a tile's spare column when it carries a terrain
/// improvement, so units and cities can keep their letters in the first
/// column: `≈` for irrigation, `⛏` for a mine, `+` for a road. When a tile
/// holds several improvements the most informative marker wins: irrigation,
/// then mine, then road. Each glyph uses a muted colour that stays readable on
/// the terrain backgrounds.
fn improvement_glyph(tile: &Tile) -> Option<(char, Color)> {
    if tile.is_irrigated() {
        Some(('≈', Color::Rgb(120, 200, 255)))
    } else if tile.is_mined() {
        Some(('⛏', Color::Rgb(235, 225, 245)))
    } else if tile.has_road() {
        Some(('+', Color::Rgb(215, 195, 135)))
    } else {
        None
    }
}

/// The civilization flag colour used to flash a selected unit's tile while it
/// awaits instruction.
pub(crate) fn civilization_color(civ: Civilization) -> Color {
    use Civilization::*;
    match civ {
        American => Color::Rgb(220, 130, 40),
        Aztec => Color::Rgb(180, 120, 80),
        Babylonian => Color::Rgb(120, 90, 180),
        Chinese => Color::Rgb(190, 60, 60),
        Egyptian => Color::Rgb(200, 170, 90),
        English => Color::Rgb(200, 60, 80),
        French => Color::Rgb(90, 110, 210),
        German => Color::Rgb(90, 90, 90),
        Greek => Color::Rgb(120, 150, 200),
        Indian => Color::Rgb(170, 120, 60),
        Mongol => Color::Rgb(130, 90, 40),
        Roman => Color::Rgb(160, 50, 50),
        Russian => Color::Rgb(120, 40, 140),
        Zulu => Color::Rgb(90, 60, 50),
    }
}

pub(crate) fn tile_style(explored: bool, terrain: Terrain) -> Style {
    if !explored {
        return Style::default()
            .fg(Color::Rgb(20, 20, 20))
            .bg(Color::Rgb(10, 10, 36));
    }
    let (fg, bg) = terrain_colors(terrain);
    Style::default().fg(fg).bg(bg)
}

/// Paint one world tile at `(x, y)` spanning `TILE_WIDTH` columns, exactly as
/// the main map does: the terrain/unit/city symbol in the first cell and a
/// blank style-carrying cell to its right. World columns wrap horizontally.
/// Returns the city's name when the tile holds an explored city so callers can
/// draw the label beneath it.
///
/// `selected_city` outlines the matching city tile; `flashing` gates the
/// selected-unit flash, which itself must be the selected unit with moves left.
/// `hover_target` (world tile, wrapped horizontally) hatches that tile with
/// `▓` so the player can see where clicking would move the selected unit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tile(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    view: &dyn GameView,
    world_x: usize,
    world_y: usize,
    map_w: usize,
    map_h: usize,
    selected_city: Option<CityId>,
    selected_unit: Option<UnitId>,
    flashing: bool,
    hover_target: Option<(usize, usize)>,
) -> Option<String> {
    let map_x = world_x % map_w;
    let hovered = hover_target == Some((map_x, world_y));
    let map_x = world_x % map_w;
    let marker = if world_y < map_h && view.explored(map_x, world_y) {
        improvement_glyph(view.tile(map_x, world_y))
    } else {
        None
    };
    let (symbol, style, city_name) = if world_y >= map_h {
        (' ', Style::default().bg(Color::Rgb(6, 6, 22)), None)
    } else {
        let tile = view.tile(map_x, world_y);
        let explored = view.explored(map_x, world_y);
        let terrain = tile.terrain;
        let mut style = tile_style(explored, terrain);

        let unit = view.units_at(map_x, world_y);
        let city = view.city_at(map_x, world_y);

        let (symbol, city_name) = if explored
            && unit.is_empty()
            && let Some(city) = city
        {
            // A city with no unit on its tile shows its population on a
            // background of the owning civilization's colour.
            (population_digit(city.population()), Some(city.name.clone()))
        } else if explored && let Some(u) = unit.first() {
            // A unit always shows its class letter ahead of the terrain, even
            // when it stands on a city tile; the city keeps its name label
            // beneath.
            (
                first_letter(u.unit_class),
                city.as_ref().map(|c| c.name.clone()),
            )
        } else if explored {
            (terrain.as_char(), None)
        } else {
            // Unexplored tiles are blank fog: the terrain letter, units and
            // cities on them must never show, even when the terminal's text
            // selection inverts the colours.
            (' ', None)
        };

        // A visible city's tile always wears its owner's civilization colour,
        // even beneath a unit standing on it: a freshly captured city changes
        // colour the instant it falls, before the conquering unit moves off.
        if explored {
            if let Some(city) = city {
                style = style.bg(civilization_color(view.civilization_of(city.owner())));
                // The selected city's tile is outlined so it stands out —
                // unless a unit covers the tile and is already underlined.
                if selected_city == Some(city.id()) && unit.is_empty() {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
            }
            // A unit (in a city or not) is painted like any other unit: its
            // letter on the tile, bold and underlined. On a city tile it sits
            // on the city's colour, so the conquest colour is visible through
            // the occupation. Its idle flash still turns the tile the flag
            // colour; on an own city that matches the background, so the pulse
            // is most visible when the unit stands on foreign ground.
            if !unit.is_empty() {
                style = style
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED);
            }
        }

        // The selected unit awaiting instruction flashes once per second: its
        // tile turns the civilization flag colour then dims back to terrain.
        if flashing
            && let Some(id) = selected_unit
            && unit
                .iter()
                .any(|u| u.id() == id && u.order() == UnitOrder::Idle && u.moves_remaining() > 0)
        {
            style = style.bg(civilization_color(view.current_player()));
        }

        (symbol, style, city_name)
    };

    let symbol = if hovered { '▓' } else { symbol };
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(&symbol.to_string());
        cell.set_style(style);
    }
    // The spare column carries the improvement watermark (≈ irrigation, + road,
    // ⛏ mine) on explored tiles; fog and the below-map gutter stay blank, and
    // hovering the tile hatches the whole tile.
    if TILE_WIDTH > 1
        && let Some(cell) = buf.cell_mut((x + 1, y))
    {
        if hovered {
            cell.set_symbol("▓");
            cell.set_style(style);
        } else if let Some((glyph, color)) = marker {
            cell.set_symbol(&glyph.to_string());
            cell.set_style(style.fg(color));
        } else {
            cell.set_symbol(" ");
            cell.set_style(style);
        }
    }
    city_name
}

fn first_letter(unit_class: UnitClass) -> char {
    format!("{unit_class:?}").chars().next().unwrap_or('?')
}

/// The digit shown on a city tile: its population, clipped to a single char.
pub(crate) fn population_digit(population: u32) -> char {
    if population >= 10 {
        '9'
    } else {
        char::from_digit(population, 10).unwrap_or('0')
    }
}

/// The yellow used for city labels.
pub(crate) const CITY_LABEL_FG: Color = Color::Rgb(255, 215, 0);

/// Set a single cell's symbol and style.
pub(crate) fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

/// Fill a horizontal run of cells between `x0` and `x1` (inclusive) on row `y`.
pub(crate) fn fill_row(buf: &mut Buffer, x0: u16, x1: u16, y: u16, symbol: &str, style: Style) {
    for x in x0..=x1 {
        set_cell(buf, x, y, symbol, style);
    }
}

/// Draw `name` centred beneath a city tile spanning the two cells that begin
/// at `tile_cx`, on the row `row_y`, in yellow.
pub(crate) fn draw_city_label(buf: &mut Buffer, tile_cx: u16, row_y: u16, name: &str) {
    // The two-cell tile is centred on its second cell; centre the name there.
    let center_col = tile_cx as isize + 1;
    let len = name.len();
    let start = center_col - (len / 2) as isize;

    for (i, ch) in name.chars().enumerate() {
        let col = start + i as isize;
        if col < 0 {
            continue;
        }
        if let Some(cell) = buf.cell_mut((col as u16, row_y)) {
            cell.set_symbol(&ch.to_string());
            cell.set_style(Style::default().fg(CITY_LABEL_FG).bold());
        }
    }
}

pub(crate) fn format_year(year: i32) -> String {
    match year.cmp(&0) {
        std::cmp::Ordering::Less => format!("{} BC", -year),
        // There is no year 0; the calendar jumps from 1 BC straight to 1 AD.
        std::cmp::Ordering::Equal => "1 AD".to_string(),
        std::cmp::Ordering::Greater => format!("{year} AD"),
    }
}
