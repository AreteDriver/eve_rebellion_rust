# Battle of Caldari Prime

A second game module for EVE Rebellion featuring the conflict between Caldari and Gallente forces over Caldari Prime.

## Overview

This module reuses the core EVE Rebellion engine (weapons, heat, spawning, AI) but with:
- **New Factions**: Caldari State vs Gallente Federation
- **New Ship Rosters**: Assault Frigates (Hawk/Harpy vs Enyo/Ishkur)
- **T3 Destroyers**: Jackdaw and Hecate (unlockable)
- **New Campaign**: 5 missions over Caldari Prime

## How to Run

From the main menu, select **"CALDARI VS GALLENTE"** to enter this module.

1. Choose your faction (Caldari or Gallente)
2. Select difficulty
3. Choose your ship
4. Play through the 5-mission campaign

## Faction Selection

The module features a split-screen faction select:
- **Left**: Caldari State (Missiles, Shields, ECM)
- **Right**: Gallente Federation (Drones, Armor, Blasters)

Your chosen faction determines:
- Player ship pool
- Enemy ship pool (automatically set to opposing faction)
- UI theme colors
- Dialogue flavor

## Ship Rosters

### Caldari Player Ships
| Ship | Class | Role | Notes |
|------|-------|------|-------|
| Hawk | Assault Frigate | Missile Boat | Starter |
| Harpy | Assault Frigate | Railgun Platform | Starter |
| Jackdaw | Tactical Destroyer | Mode-Switching | Unlocked Mission 4 |

### Gallente Player Ships
| Ship | Class | Role | Notes |
|------|-------|------|-------|
| Enyo | Assault Frigate | Blaster Brawler | Starter |
| Ishkur | Assault Frigate | Drone Boat | Starter |
| Hecate | Tactical Destroyer | Mode-Switching | Unlocked Mission 4 |

## Mission Chain

1. **Orbital Skirmish** - Tutorial, first contact
2. **Urban Firefight** - Combat over cities
3. **Fleet Interdiction** - Intercept reinforcements
4. **Escalation Point** - T3 destroyers enter (unlocks T3)
5. **Decisive Push** - Final battle for superiority

## Epilogue: Shiigeru Nightmare

The Caldari arc includes an endless survival epilogue:

**"Final Directive: Shiigeru"** - Endless survival aboard the dying Leviathan-class titan as it plummets toward Caldari Prime.

### Mechanics
- **Escalating Difficulty**: Waves spawn faster over time, enemies get stronger
- **Mini-Bosses**: Security Chief, Weapons Officer, Drone Swarm, Bridge Commander
- **Hull Integrity**: Visual tension element that decreases over time
- **High Scores**: Track best wave reached and survival time

### Mini-Boss Rotation
| Wave Pattern | Boss Type | Health |
|--------------|-----------|--------|
| Every 4th wave | Drone Swarm | 200 |
| Wave % 4 = 1 | Security Chief | 300 |
| Wave % 4 = 2 | Weapons Officer | 400 |
| Wave 20+ | Bridge Commander | 600 |

## Directory Structure

```
games/caldari_gallente/
├── config/
│   └── module.json     # Module manifest with ship pools, missions
├── assets/             # Module-specific assets (emblems, backgrounds)
├── ui/                 # UI customizations
├── ships/              # Ship configuration overrides
├── missions/           # Mission-specific data
└── README.md           # This file
```

## Adding New Ships

1. Add ship definition to `config/module.json` under the appropriate faction's `playerShips` or `enemyShips` array
2. Include the EVE type ID, stats, and spawn weight (for enemies)
3. The engine will automatically load sprites from the shared sprite cache

## Adding New Missions

1. Add mission definition to `config/module.json` under `missions`
2. Add boss definition if applicable under `bosses`
3. The engine's existing spawning and boss systems handle the rest

## Configuration Reference

See `config/module.json` for the complete module manifest including:
- `moduleId`: Unique identifier
- `factions`: Faction definitions with colors and doctrine
- `shipPools`: Player and enemy ships per faction
- `missions`: Campaign mission chain
- `bosses`: Boss definitions with health/phases

## Technical Notes

- Reuses existing spawning system (just swaps ship type pools)
- Reuses existing weapons/heat mechanics unchanged
- Reuses existing AI behaviors unchanged
- UI theme colors are applied via faction selection
- T3 mode-switching is simplified: modes only change between missions

## Credits

Part of EVE Rebellion by the EVE Rebellion Team.
Inspired by the Battle of Caldari Prime (YC115) from EVE Online lore.
