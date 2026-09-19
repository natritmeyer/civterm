# civterm — UML diagrams

Generated from the source in `src/` (`lib.rs`: `model`, `game_engine`, `tui`, `utils`).
Conventions follow `ARCHITECTURE.md` / `AGENTS.md`: one-way layering
(`game_engine` + `tui` depend on `model`; `model` never references `game_engine`),
`Engine::submit(Command) -> Vec<Event>` as the only mutation path, and
`GameView` as the read-only render interface.

All diagrams are [Mermaid](https://mermaid.js.org/) fenced blocks so they render
natively on GitHub with no extra tooling. Sources:

| # | File | UML kind | What it shows |
|---|------|----------|---------------|
| 1 | [01-architecture-packages.md](01-architecture-packages.md) | Package / layer (flowchart) | Crate layers, one-way dependencies, `Command`/`Event`/`GameView` flows |
| 2 | [02-model-domain.md](02-model-domain.md) | Class | `model`: units, cities, civilizations, cartography, geography, advancements |
| 3 | [03-engine-structure.md](03-engine-structure.md) | Class | `game_engine`: `Engine`, `Game`, `Player`, `Command`, `Event`, `GameView`, save types |
| 4 | [04-tui-structure.md](04-tui-structure.md) | Class | `tui`: `App`, `GameScreen`, dialogs, pickers, overlays |
| 5 | [05-command-flow-sequence.md](05-command-flow-sequence.md) | Sequence | Keystroke/mouse → `Command` → `Engine::submit` → `Event` → render (move/combat/capture path) |
| 6 | [06-turn-loop-sequence.md](06-turn-loop-sequence.md) | Sequence | `EndTurn` whole-round resolution + deterministic rival AI |
| 7 | [07-app-state.md](07-app-state.md) | State machine | `App` `Phase` setup flow + `Playing` modal priority |

Regenerate by re-reading `src/model`, `src/game_engine`, `src/tui`, `src/utils`
and updating the Mermaid blocks in place.
