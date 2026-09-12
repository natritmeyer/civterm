# civterm

A Civ game in your terminal

![Splash screen](/docs/civterm_splash_screen_for_social.png)

## Repo info

- [![Made with Zed](https://img.shields.io/badge/Made%20with-Zed-084CCF?logo=zed)](https://zed.dev) [![Made with OpenCode](https://img.shields.io/badge/Made%20with-OpenCode-8A2BE2?logo=zed)](https://opencode.ai) [![Licence](https://img.shields.io/github/license/natritmeyer/civterm)](<>)
- [![Language](https://img.shields.io/github/languages/top/natritmeyer/civterm)](<>) [![Repo size](https://img.shields.io/github/repo-size/natritmeyer/civterm)](<>)
- [![Last commit](https://img.shields.io/github/last-commit/natritmeyer/civterm)](<>) [![Build status](https://img.shields.io/github/actions/workflow/status/natritmeyer/civterm/ci.yml?label=build)](https://github.com/natritmeyer/civterm/actions) [![Tests](https://img.shields.io/badge/tests-481%20passing-brightgreen)](<>)

## Setup

Run `make setup` once after cloning to install the pre-commit hook (it runs
`make build` — fmt, clippy, build, and all tests — before every commit):

```
make setup
```

Skip the hook for a single commit with `git commit --no-verify`.

## What this is about

- A bit of fun, recreating what I remember from my early teen years in the mid 90s of the MacOS version of Civilization
- I don't touch the code - I summon it from OpenCode's Big Pickle LLM within the Zed text editor
- Coded in rust (I don't know rust)
- Written as a TUI (I've never written a TUI and I've never used Ratatui)

# Screenshots of progress...

### We have units!

![Flashing unit](/docs/flashing_unit.gif)

### We can found new cities!

![City](/docs/city.png)

### We can drag the map around!

![Dragging the map](/docs/map_drag.gif)

### We can see what's happening in cities!

![City view](/docs/initial_city_view.png)

### Cities can now make units and improvements!

![City with units and improvements](/docs/city_with_improvement_and_unit.png)

### We can sail the seven seas!

![Trireme](/docs/trireme.png)

### We can irrigate, mine, and build roads on tiles!

![Settler work](/docs/settler_work.png)

### Civilizations can be conquered!

![Civilizations can be conquered](/docs/war.png)

# Inspiration

> /me On a nostalgia binge, running an emulator...

- **Son**: Woah dad, that's so cool!
- **Me**: Yeah, this was the best game ever. I wasted most of my teen years playing it.
- **Son**: What game is it?
- **Me**: Civilization 1
- **Son**: What did you play it on? Was it on the Nintendo Switch?
- **Me**: Pfft. I used to play it on my beloved Apple Macintosh LC-475. The greatest machine ever. This thing here is just an emulator and I can't save the game :/
- **Me**: ...hmm...
- **Me**: Mabye I could...
- **Son**: What?
- **Me**: Maybe I should rewrite the game with AI...
- **Me**: ...and I could make it run in the terminal!
- **Son**: What's the terminal?
- **Me**: I have failed you.
