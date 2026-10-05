# 6 — Turn loop / rival AI sequence diagram

`Command::EndTurn` resolves the whole round on the engine (`turns.rs` +
`rival_player_engine.rs`): each rival acts in player order, then control returns
to the human and the turn increments once. The AI's decisions are deterministic —
it never consults `Engine.rng` to choose what to research, build, settle, march
or fight (the only RNG draw a rival triggers is ordinary combat resolution,
shared with the player) — and every rival step goes through the ordinary
`move_unit` path, so movement/reveal/transport/combat invariants hold and each
landing is recorded as `RivalMotion` for the TUI replay, combat landings flagged
so the replay flashes the tile the fight was fought on. A rival attack starts
with a war declaration: when a military unit stands beside a human unit or city
whose strongest defender it outguns (`attacker_power > defender_power`, or an
undefended city), it declares war, records a transient `RivalWar`, and the TUI
announces it in an OK window after the round. The odds it weighs are
`win_chance_percent`, which models the race `resolve_combat` actually fights
(`a^n / (a^n + d^n)` over `HIT_POINTS`, not the linear share `a / (a + d)`).

A siege engine is the exception, and it is a policy exception rather than a
weaker one. It has no quarrel with the garrison, so the power comparison does
not apply to it at all: it bombards a walled city it could never outfight,
which is the whole reason to build one. It does not declare a war to do so — a
bombardment strips walls but takes nothing — so siege only ever follows a war
already declared. Once the walls are down and no soldiers hold the tile, the
same engine walks in and takes the city, finishing the conquest.

```mermaid
sequenceDiagram
    actor Human
    participant App as App
    participant Eng as Engine::submit/end_turn
    participant Tur as turns.rs
    participant Riv as rival_player_engine.rs
    participant Mov as movement.rs/combat.rs
    participant Gam as Game

    Human->>App: Enter (end turn)
    App->>Eng: submit(EndTurn)
    Eng->>Tur: end_turn()

    loop round-robin until back to human
        Tur->>Tur: advance_to_next_player()<br/>(skip eliminated)
        Tur->>Tur: begin_turn(current)
        Tur->>Gam: restore moves / advance terrain work<br/>process_cities(current)<br/>process_research(current)
        alt current is rival
            Tur->>Riv: run_rival_turn()
            Riv->>Riv: rival_research()<br/>first RIVAL_RESEARCH_PRIORITY<br/>that can_research, else cheapest
            Riv->>Riv: rival_production()<br/>garrison need → cheapest military<br/>cities<4 → Settler else cheapest unit
            Riv->>Riv: rival_units(): settlers then garrisons<br/>then rival_warfare()
            loop each rival unit step
                Riv->>Riv: best_settlement_site()<br/>open(≤3 overlap) → acceptable(≤6)<br/>→ frontier edge → crowded
                alt winnable fight beside unit
                    Riv->>Riv: rival_best_attack() → (Direction, starts_war)<br/>attacker_power > defender_power<br/>win_chance_percent ≥ RIVAL_MINIMUM_WIN_CHANCE<br/>(models the race to 10, not a/(a+d))
                    Riv->>Riv: declare_war(human) + push RivalWar
                else siege engine beside a walled city
                    Riv->>Riv: skips the power test — a bombardment<br/>never meets the garrison, so odds do not apply.<br/>Worth a turn while walls stand; then only<br/>if no soldiers hold the tile
                    Riv->>Riv: starts_war = false:<br/>siege follows a war already declared,<br/>it is no reason to begin one
                end
                Riv->>Mov: move_unit(step)
                Mov->>Gam: spend_moves + reveal_tiles_at<br/>+ sync_cargo
                Mov->>Riv: push RivalMotion{unit, from, to,<br/>battle} (owner != human)<br/>no motion for a bombardment
            end
        end
    end

    Tur->>Tur: turn += 1
    Tur->>App: Vec~Event~ ('begins turn N')
    App->>App: drain_rival_wars()<br/>→ WarDialog (OK window) per declaration
    App->>App: drain_rival_motion()<br/>filter: explored(from) OR explored(to)
    App->>App: RivalMoveAnimation{frames}<br/>2 × 300ms sub-phases per step<br/>battle frame → 💥 on arrival
    App->>App: if new advancement → ResearchDialog
    App-->>Human: replay plays (input swallowed,<br/>camera pans if step leaves central 70%)
```

## Settlement tiers (`best_settlement_site`)

Measured against the union of **all** civilisations' 21-tile working grids, deterministic
(best score → nearest to settler → smallest tile):

1. **Open** — shares at most 3 tiles (≥ Chebyshev 4 off every city).
2. **Acceptable** — shares at most 6 tiles (≥ Chebyshev 3 out).
3. **Frontier** — nearest explored land tile bordering unexplored ground, so reveals open new country.
4. **Crowded** — best-scored tile, only when the map is fully explored.

Sites are only ever chosen from explored tiles. Score is `2 × resources + food`.
