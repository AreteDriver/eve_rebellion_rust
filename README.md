# EVE Rebellion

[![Version](https://img.shields.io/badge/version-1.4.0-blue.svg)](https://github.com/AreteDriver/eve_rebellion_rust/releases)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Web-purple.svg)]()

**EVE Rebellion** puts you in the cockpit during the Minmatar's greatest hour - the Elder Fleet invasion of YC110. Rise from a rookie pilot in a rusty Rifter to an ace liberator in a Jaguar, freeing your people from centuries of Amarr slavery.

## The Story

In YC110, the Elder Fleet emerged from decades of hiding to liberate the Minmatar people still enslaved in Amarr space. You are a young pilot who answers their call, joining the greatest liberation in New Eden's history.

Guided by a tribal Elder who speaks for all seven tribes, you'll fight through Amarr patrols, break slave colony defenses, and face increasingly powerful warships - culminating in a desperate assault on an Avatar-class Titan.

**Every soul you liberate matters. Every chain you break echoes through history.**

## Features

- **Vertical Shooter Action** - Fast-paced shmup gameplay with EVE Online ships and lore
- **Liberation Campaign** - 13 stages across 3 acts telling the Elder Fleet story
- **Heat & Combo System** - Push your weapons to the limit for score multipliers
- **Berserk Mode** - Fill your rage meter to unleash devastating power
- **Ship Progression** - Earn your way from Rifter to Wolf to Jaguar
- **Authentic EVE Visuals** - Ships rendered from official EVE Online assets

## Controls

### Keyboard
- **WASD / Arrow Keys** - Move
- **Space** - Fire
- **B** - Activate Berserk Mode (when meter full)

### Controller (Xbox/PlayStation)
- **Left Stick** - Move
- **Right Trigger** - Fire
- **Y / Triangle** - Activate Berserk Mode

## Building

Requires Rust 1.75+ and Bevy 0.15.

```bash
cargo build --release
cargo run --release
```

## Project Structure

```
eve_rebellion_rust/
├── src/
│   ├── main.rs           # Entry point
│   ├── core/             # Game states, events, resources
│   ├── entities/         # Player, enemies, projectiles, collectibles
│   ├── systems/          # Game logic (joystick, scoring, combat)
│   ├── ui/               # HUD and menus
│   └── assets/           # Asset loading
├── assets/               # Sprites, icons, audio
├── config/               # JSON configuration
│   ├── enemies_amarr.json    # Amarr enemy definitions
│   ├── bosses_campaign.json  # 13-boss campaign structure
│   └── dialogue_elder.json   # Elder mentor dialogue
└── docs/
    └── NARRATIVE_DESIGN.md   # Story bible and design notes
```

## Credits

- **EVE Online** is a trademark of CCP hf.
- Ship images provided via CCP's Image Server under their community use guidelines
- This is a fan project, not affiliated with or endorsed by CCP

## License

This project is licensed under the MIT License.

---

*"We are the storm that breaks the chains. We are the Minmatar."*
