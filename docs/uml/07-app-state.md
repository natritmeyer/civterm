# 7 — `App` state machine

Source: `src/tui/app/mod.rs` (`Phase`), `setup.rs`, `playing.rs`, `dialogs.rs`,
`pickers.rs`, `mouse.rs`. `App::draw` matches `phase`; `handle_key` dispatches
per phase. `Playing` keeps modal priority: save prompt swallows everything, then
research → diplomacy → work picker → production picker → rival replay.

```mermaid
stateDiagram-v2
    [*] --> Menu : App::new()

    Menu --> ChoosingCiv : N / Enter (New Game)
    Menu --> Menu : L (Load Saved Game → SaveLoadPrompt.Load)
    Menu --> [*] : Q / Quit

    ChoosingCiv --> ChoosingCompetition : Enter (civ chosen)
    ChoosingCiv --> Menu : Esc / q

    ChoosingCompetition --> ChoosingDifficulty : Enter (rivals 1..7)
    ChoosingCompetition --> ChoosingCiv : Esc

    ChoosingDifficulty --> ReadyToStart : Enter (Easy/Normal/Hard)
    ChoosingDifficulty --> ChoosingCompetition : Esc

    ReadyToStart --> Playing : S / Enter (start_game())
    ReadyToStart --> ChoosingDifficulty : Esc
    ReadyToStart --> [*] : Q (Quit)

    state Playing {
        [*] --> Exploring
        Exploring --> CityWindowOpen : select city (click / key)
        CityWindowOpen --> ProductionPickerOpen : change production
        ProductionPickerOpen --> CityWindowOpen : save / cancel
        CityWindowOpen --> Exploring : close (Esc / ✕)

        Exploring --> WorkPickerOpen : w on Settler
        WorkPickerOpen --> Exploring : save Work{imp} / cancel

        Exploring --> ResearchDialogOpen : advancement discovered
        ResearchDialogOpen --> Exploring : confirm SetResearchTarget

        Exploring --> DiplomacyOpen : first contact / peaceful-block
        DiplomacyOpen --> Exploring : DeclareWar (+retry move) / MakePeace / cancel

        Exploring --> SavePromptOpen : S (save) 
        SavePromptOpen --> Exploring : save_game() / cancel

        Exploring --> RivalReplay : EndTurn → drain_rival_motion()
        RivalReplay --> Exploring : animation complete
    }

    Playing --> Menu : q (quit to menu, via confirm path)
    Playing --> Playing : Enter (EndTurn, stays in Playing)
```

## Modal input priority (`handle_playing_key` / `handle_mouse`)

1. `save_prompt` open → all keys/mouse go to the prompt.
2. `research_dialog` → research keys only.
3. `diplomacy` → diplomacy keys only.
4. `work_picker_open` → work picker keys only.
5. `production_picker_open` → production picker keys only.
6. `rival_animation` active → swallows game input (modals still capture), pans camera.
7. Otherwise: unit keys (`arrows/hjkl/yubn`, `Tab` cycle, `space` sentry-wait),
   `v` found city, `w` work, `c` cancel order, `e` toggle events, `?` help,
   `S` save, `Enter` end turn, `Esc` deselect/close.

`SaveLoadPrompt` renders topmost in every phase (`draw` paints it last).
Default save path is `civterm.civ`.
