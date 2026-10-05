# 5 — Command flow sequence diagram

One keystroke/mouse action → one `Command` → `Engine::submit` → `Vec<Event>` →
`App::record_events` + overlay triggers → next `GameScreen` frame. Shown here is
the richest path: a combat move (`movement.rs` → `combat.rs`), including the
city-capture early return and the reveal invariant. The combat tail has two
shapes: an ordinary fight, and a siege engine's bombardment, which never meets
the garrison at all. A rival attack takes this same path (after its
`declare_war`): its landing is recorded as a `RivalMotion { battle: true }`
before the fight resolves, so the replay can flash the tile it struck. A
bombardment records nothing, because the engine never changes tile.

```mermaid
sequenceDiagram
    actor Human
    participant App as App (tui/app)
    participant Eng as Engine::submit
    participant Mov as movement.rs
    participant Cbt as combat.rs
    participant Gam as Game (map/units/cities)
    participant Scr as GameScreen

    Human->>App: key / click (e.g. arrow / hjkl / map click)
    App->>App: move_selected_unit(dir)
    App->>Eng: submit(Move{unit, direction})

    Eng->>Mov: move_unit(unit, direction)
    Mov->>Mov: ensure_can_move()<br/>(owned, moves left,<br/>on-map, medium, peaceful)
    alt cannot cross land/sea border
        Mov->>Mov: try_board()<br/>(adjacent friendly transport,<br/>free berth, no move cost)
        Mov->>Gam: meet_contacts_within() + reveal_tiles_at()
    else siege engine on a held tile<br/>(no city to bombard)
        Mov->>Mov: Err(CannotAttackUnits)
    end
    Mov->>Mov: meet_contacts_within(dest)
    Mov->>Gam: enemies_present?(dest)<br/>(cargo skipped)

    alt enemies present, or a city with<br/>improvements still standing
        Mov->>Cbt: resolve_move_combat(attacker, dest)
        alt attacker is a siege engine
            Cbt->>Cbt: bombard_city()<br/>(never meets the garrison:<br/>no fight, no capture)
            Cbt->>Gam: remove one improvement,<br/>CityWalls first<br/>(strips the walls bonus)
            Cbt->>Cbt: spend_turn() + Event::between
            Cbt->>Cbt: no RivalMotion —<br/>the engine never changes tile
        else ordinary fight
            Cbt->>Gam: select_defender(dest)<br/>(max defence power)
            Cbt->>Cbt: resolve_combat()<br/>(race to HIT_POINTS 10:<br/>hit lands a/(a+d) per round,<br/>vet/terrain/city/wall/fort mods, rng)
            alt attacker wins
                Cbt->>Gam: remove_unit(defender) + disband_cargo_of()
                Cbt->>Gam: advance + disembark +<br/>spend_turn + promote + sync_cargo
                Cbt->>Gam: reveal_tiles_at(winner, dest)
                Cbt->>Gam: eliminate_if_annihilated(loser)
            else defender wins
                Cbt->>Gam: remove_unit(attacker)
                Cbt->>Gam: eliminate_if_annihilated(attacker owner)
            end
        end
    else undefended at-war foreign city on dest
        Mov->>Cbt: capture_city(id, unit, dest)
        Cbt->>Gam: disband_units_homed_to() + disband_cargo_of()
        Cbt->>Gam: change_owner(current) + advance +<br/>disembark + spend + reveal
        Cbt->>Gam: eliminate_if_cityless(old owner)
    else plain move
        Mov->>Gam: location = dest, disembark()
        Mov->>Gam: spend_moves(terrain cost)
        Mov->>Gam: reveal_tiles_at(owner, dest)
        Mov->>Gam: sync_cargo(carrier)
        Mov->>Mov: record RivalMotion (owner != human)
    end

    Mov-->>Eng: Ok / MoveError
    Eng->>Eng: push Event::new(...) messages
    Eng-->>App: Vec~Event~
    App->>App: record_events() (keep last 5)
    App->>App: if 'attacks' → battle_animation<br/>if 'meet for the first time' → Diplomacy(Contact)<br/>if 'occupied by a civilization at peace' → Diplomacy(Movement)
    App->>Scr: draw(&dyn GameView, events, overlays)
    Scr-->>Human: map + stats + focus + event log
```

## Invariants on this path

- Every tile landing reveals for the mover (`reveal_tiles_at`); combat advances
  and captures reveal for themselves (separate early returns).
- Movement spends the terrain cost (`spend_moves(cost)`), never `spend_turn()`;
  only combat/capture spend the whole turn. Boarding/disembarking cost nothing
  but still reveal.
- Cargo is inert: omitted from `units_at`, skipped by `select_defender` /
  `enemies_present` / `ensure_peaceful_passage`.
- `UnitClass::attacks_units` / `sieges` split the two tail shapes. A siege
  engine is routed to the tail whenever the destination holds an enemy city
  **with improvements standing** — otherwise an undefended but walled city
  would change hands without a shot. With the city stripped and no soldiers on
  it, it falls through to the ordinary capture path instead: it cannot fight
  its way in, but it need not once nothing is left to defend.
- `bombard_city` spends the whole turn on one improvement, `CityWalls` first.
  Stripping the walls removes `CITY_WALLS_DEFENSE_BONUS` from every defender in
  the city, which is the point: a losing attacker is removed outright and no
  wound state survives between fights, so a defended city cannot be ground
  down by attrition.
- Fortify / Sentry / Work / CancelOrder / FoundCity / SetProductionTarget /
  DeclareWar / MakePeace / SetResearchTarget follow the same
  `App → submit → concern → Events → record → draw` skeleton with their own
  guards (see `03-engine-structure.md` dispatch table).
