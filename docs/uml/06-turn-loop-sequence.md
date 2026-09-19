# 6 — Turn loop / rival AI sequence diagram

`Command::EndTurn` resolves the whole round on the engine (`turns.rs` +
`rival_player_engine.rs`): each rival acts in player order, then control returns
to the human and the turn increments once. The AI is deterministic — it never
draws from `Engine.rng` — and every rival step goes through the ordinary
`move_unit` path, so movement/reveal/transport/combat invariants hold and each
landing is recorded as `RivalMotion` for the TUI replay.

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
            Riv->>Riv: rival_units(): settlers then garrisons
            loop each rival unit step
                Riv->>Riv: best_settlement_site()<br/>open(≤3 overlap) → acceptable(≤6)<br/>→ frontier edge → crowded
                Riv->>Mov: move_unit(step)<br/>(plain tail or board only)
                Mov->>Gam: spend_moves + reveal_tiles_at<br/>+ sync_cargo
                Mov->>Riv: push RivalMotion{unit, from, to}<br/>(owner != human)
            end
        end
    end

    Tur->>Tur: turn += 1
    Tur->>App: Vec~Event~ ('begins turn N')
    App->>App: drain_rival_motion()<br/>filter: explored(from) OR explored(to)
    App->>App: RivalMoveAnimation{frames}<br/>2 × 300ms sub-phases per step
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
