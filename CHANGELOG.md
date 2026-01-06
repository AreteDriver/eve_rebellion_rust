# Changelog

All notable changes to EVE Rebellion will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- The Last Stand mode (fixed-platform titan defense)
- Ship ability system with cooldowns
- Drone entities and heat display
- Faction-aware dialogue system

### Fixed
- Cargo fmt and clippy dead_code warning
- Prevent regular player/HUD spawn during Last Stand mode

## [1.5.1] - 2025-01-05

### Added
- CLAUDE.md project instructions
- itch.io badge to README
- Platform support documentation

### Fixed
- Clippy lint fixes (is_multiple_of, Range::contains)
- Cargo fmt formatting
- itch.io deploy continue-on-error

## [1.5.0] - 2025-01-04

### Added
- Automated itch.io deployment on release
- WASM build job in release workflow
- Linux, Windows, and Web builds pushed to itch.io

### Fixed
- CG boss projectiles not moving (use ProjectilePhysics)
- CG boss sprites (use faction-appropriate ship type IDs)
- Sprite rotation (bosses face player)
- Available sprites for CG bosses (no missing textures)

## [1.4.2] - 2025-01-03

### Added
- Combo timer bar below combo counter
- Settings persistence to disk (audio, screen shake, rumble)
- Rumble intensity slider in pause menu

## [1.4.1] - 2025-01-03

### Added
- Controller rumble/haptic feedback
- Pause menu volume sliders (Master, Music, SFX)
- Screen shake intensity slider
- Elder Fleet endless mode wave/boss announcements
- Nightmare mode announcements

### Fixed
- CG mission advancement timing

## [1.4.0] - 2025-01-02

### Added
- Critical Hit System with visual feedback
- Battle of Caldari Prime Campaign (5 missions)
- Shiigeru Nightmare Mode (endless survival)
- Mode select screen for Caldari
- Nightmare HUD with wave/time/kills/hull

## [1.3.0] - 2025-01-01

### Added
- Endless Mode survival gameplay
- Visual powerup status bar with timer bars
- Redesigned ship selection UI with stat bars
- More sound effects (missiles, waves, powerups)
- Berserk system redesign with meter

## [1.2.1] - 2024-12-31

### Added
- Web/WASM build support
- GitHub Pages deployment workflow

## [1.2.0] - 2024-12-30

### Added
- Initial release with Elder Fleet campaign
- 4 factions: Minmatar, Amarr, Caldari, Gallente
- 13-stage campaign with boss battles
- Procedural audio system
- Ship unlocks and progression

[Unreleased]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.5.1...HEAD
[1.5.1]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.2.1...v1.3.0
[1.2.1]: https://github.com/AreteDriver/eve_rebellion_rust/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/AreteDriver/eve_rebellion_rust/releases/tag/v1.2.0
