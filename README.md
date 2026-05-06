# Heavy Water

A 3D open-world sci-fi action RPG built with [Bevy 0.15](https://bevyengine.org/) and Rust.
Hand-authored 14-chapter campaign across the lives of four robotic factions —
**Synthetic**, **Mechanoid**, **Insectoid**, **Swarm**, plus the **Animatons** and the
**Charred** — with chassis customization, recruitable companions, weapon/armor mods,
diegetic radio chatter narrative, and discoverable easter-egg upgrades on every map.

You play **Amp**, the firstborn Synthetic, awakening into a ruined world and gathering
your scattered brothers and sisters across hostile biomes ranging from the InsectoidMetropolis
to Char's Domain to the StarfallArena.

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| LMB | Fire / Beam Sabre slash |
| V/B | Melee |
| T | Toggle Beam Sabre (must be unlocked in Ch.1) |
| 7/8/9/0 | Special weapons |
| F | Parry · Q Dodge · Space Jump/Jetpack |
| C | Crafting |
| M | Motorcycle (requires blueprint, Ch.1) |
| J | Jet (requires blueprint, Ch.2) |
| E | Chassis Editor (from chapter select) |

## Chapters

1. **Amp!** — Awakening in the dust. Beam Sabre + Motorcycle blueprint. Aria, Valor recruited.
2. **The Plague** — Three years lost. Missile Launcher + Jet blueprint. Volt, Chroma recruited.
3. **A Tale of 3 Sisters** — Lunar Labs memory. Daria, Prima recruited.
4. **A Tale of 4 Brothers** — Brothers' fortress. Piercing Rounds. Apollo recruited.
5. **StarCity Soldier** — Star City sentinel. Reactive Plating. Atlas recruited.
6. **Char's Domain** — Wolf and Tiger Animatons. Coolant Weave.
7. **The Swarm** — Brutus, Nero, King Cygnus.
8. **Ruination** — Queen Cygni rises.
9. **Peaceful DNA** — Theta's gift; Ion is born.
10. **The Land of Ashur** — Helios stands.
11. **A Village Called Earth** — Caliguon and the Vanguard.
12. **Evolution or Eradication** — Hybrid Omega + Cygni Awakened.
13. **A Calm Winter Night** — Selene, the Winter Stalker.
14. **Starfall** — The Star, the Swarm, and the Six.

## Factions

| Faction | Color | Examples |
|---------|-------|----------|
| Synthetic | Cyan | Amp, Atlas, Volt, Chroma, Daria, Prima, Theta, Aria, Valor |
| Mechanoid | Orange | Apollo, Saturn, Mercury, Axe, Octavius, Helios, Selene, Ion |
| Insectoid | Lime | Aracnoid Queen, Punisher, Insectoid General, Formic Avatar |
| Swarm | Magenta | Cygnus, Cygni, Brutus, Nero, Minerva, Caliguon |
| Animaton | Gold | Wolf, Tiger, last Animaton |
| Charred | Red | Char Harvester, Charred Captain |

---

## Quick Start

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable (1.75+) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Cargo | bundled with Rust | — |
| Git | any | system package manager |

On **macOS** you also need the Xcode Command Line Tools:
```sh
xcode-select --install
```

On **Linux** (Ubuntu/Debian) install Bevy's system dependencies:
```sh
sudo apt-get install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

### Clone & Run

```sh
git clone <repo-url>
cd Heavy-Water-2

# Development (fast iteration — uses dynamic linking)
cargo run --features dynamic

# Release build (optimised, stripped binary)
cargo run --release
```

The first compile takes 2–4 minutes. Subsequent incremental builds with `--features dynamic` are typically under 5 seconds.

### Keyboard Controls

| Key | Action |
|-----|--------|
| `WASD` | Move |
| Mouse | Look (cursor locked) |
| `Shift` | Sprint (drains stamina) |
| `Space` | Jump / hold for Jetpack |
| `LMB` | Fire active weapon / Beam Sabre slash |
| `1–6` | Select primary weapon |
| `7` | Homing Missile |
| `8` | Energy Burst (3-bolt pierce) |
| `9` | Bomb (gravity arc, 12u AoE) |
| `0` | Combat Drone burst |
| `R` | Reload |
| `V` | Light melee combo (Jab → Cross → Uppercut) |
| `B` | Heavy melee combo (Slam → Sweep) |
| `T` | Toggle Beam Sabre (LMB to slash while active) |
| `Q` | Dodge roll (costs stamina, grants invulnerability) |
| `F` | Parry (0.2s window, blocks all damage) |
| `[ ]` | Cycle elemental armor infusion |
| `C` | Toggle crafting panel |
| `F5` | Manual save |
| `Esc` / game over → `R` | Return to main menu |

---

## Project Structure

```
Heavy-Water-2/
├── Cargo.toml                  # Dependencies & build profiles
├── README.md
└── src/
    ├── main.rs                 # App bootstrap, plugin registration
    ├── state.rs                # AppState enum (MainMenu / Playing / Paused / GameOver)
    ├── events.rs               # All 25+ custom event types + EventsPlugin
    ├── damage.rs               # DamageInfo, Health, Damageable, apply_damage()
    ├── resources.rs            # WaveInfo, GameSettings, PlayerScore, CameraShake, UiMessage
    ├── components/
    │   ├── mod.rs
    │   ├── player.rs           # PlayerStats, PlayerMovement, JetpackState, DodgeState, ParryState
    │   ├── enemy.rs            # Enemy, EnemyConfig, EnemyStateMachine, BossEnemy
    │   ├── weapon.rs           # Weapon, WeaponInventory, SpecialWeapon, BeamSabre, Projectile, MeleeCombo
    │   ├── armor.rs            # ArmorSet, ArmorPiece, ElementType
    │   ├── inventory.rs        # Inventory, ItemDefinition, all_items()
    │   ├── companion.rs        # Companion, CompanionKind, CompanionProjectile
    │   └── world.rs            # WorldGeometry, Building, Chest, WorldLoot, NeonLight
    ├── plugins/
    │   ├── mod.rs
    │   ├── player_plugin.rs    # Input, movement, dodge, parry, jetpack, camera shake, level-up
    │   ├── weapon_plugin.rs    # Fire systems, special weapons, beam sabre, melee, VFX, particles
    │   ├── enemy_plugin.rs     # Wave timer, spawning, AI, loot drops, boss encounters
    │   ├── world_plugin.rs     # Procedural city generation (8 zone types)
    │   ├── chest_plugin.rs     # Loot chest interaction
    │   ├── companion_plugin.rs # Companion follow, combat, healing
    │   ├── armor_plugin.rs     # Armor stat bonuses, elemental switching
    │   ├── crafting_plugin.rs  # Recipe system, craft queue
    │   ├── ui_plugin.rs        # HUD, main menu, game-over, damage vignette, crafting panel
    │   └── save_plugin.rs      # JSON checkpoint save/load (F5 / autosave every 30s)
    └── robots/
        ├── mod.rs
        ├── designer.rs         # RobotStyle — 20+ procedural parameters
        ├── factory.rs          # Spawns robot mesh hierarchy from RobotStyle
        └── presets.rs          # 12 named robot presets (ScoutPrime, TankTitan, HybridOmega…)
```

---

## Developer Guide

### Architecture Overview

The game uses Bevy's **ECS (Entity Component System)** throughout. The three key patterns to understand:

1. **State-gated systems** — every gameplay system is wrapped in `.run_if(in_state(AppState::Playing))` so it only ticks when the game is actually running.
2. **Event-driven communication** — systems never query each other directly. They fire events (`EnemyKilledEvent`, `PlayerDamagedEvent`, etc.) that other systems react to. All events are declared in [events.rs](src/events.rs) and registered in `EventsPlugin`.
3. **Component composition** — entities are built from small, single-purpose components. The player entity has ~12 components stapled together in `spawn_player()`.

### Adding a New System

1. Write the system function in the appropriate plugin file.
2. Add it to the plugin's `.add_systems(Update, your_system.run_if(in_state(AppState::Playing)))` call.
3. If the system needs to run in a specific order relative to others, use `.chain()` or `.before()`/`.after()`.

```rust
// Example: new system in weapon_plugin.rs
fn my_new_system(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
) {
    // ...
}

// In WeaponPlugin::build():
app.add_systems(Update, my_new_system.run_if(in_state(AppState::Playing)));
```

### Adding a New Event

1. Declare the struct in [events.rs](src/events.rs):
```rust
#[derive(Event, Debug)]
pub struct MyNewEvent {
    pub value: f32,
}
```
2. Register it in `EventsPlugin::build()`:
```rust
.add_event::<MyNewEvent>()
```
3. Fire it with `EventWriter<MyNewEvent>`, read it with `EventReader<MyNewEvent>`.

### Adding a New Component

Add the struct to the relevant file in [src/components/](src/components/). Everything in that module is re-exported via `components/mod.rs`, so it's immediately available via `use crate::components::*`.

```rust
// In src/components/player.rs
#[derive(Component, Debug, Clone)]
pub struct MyNewComponent {
    pub value: f32,
}
```

### Adding a New Enemy Type

1. Add the variant to `EnemyType` in [components/enemy.rs](src/components/enemy.rs).
2. Add its config inside `EnemyConfig::for_type()`.
3. Add a spawn weight in `select_enemy_type()` in [enemy_plugin.rs](src/plugins/enemy_plugin.rs).
4. Either map it to an existing robot preset in `spawn_enemy_entity()`, or create a new preset in [robots/presets.rs](src/robots/presets.rs).

### Adding a New Weapon

**Primary weapons (1–6):**
1. Add the variant to `WeaponType` in [components/weapon.rs](src/components/weapon.rs).
2. Add its stats in `Weapon::new()` (damage, fire rate, ammo, pellets, spread, etc.).
3. Add it to the `WeaponInventory::default()` slots array.
4. Map it to a mesh/material in `weapon_fire_system()` inside [weapon_plugin.rs](src/plugins/weapon_plugin.rs).
5. Add a slot label to the HUD weapon bar in [ui_plugin.rs](src/plugins/ui_plugin.rs).

**Special weapons (7–0):**
Add a new branch in `special_weapon_system()` in [weapon_plugin.rs](src/plugins/weapon_plugin.rs). Special weapons reuse the `Projectile` component — just configure damage, speed, explosion radius, and mesh size.

### Adding a New Crafting Recipe

Open [plugins/crafting_plugin.rs](src/plugins/crafting_plugin.rs) and add an entry to `all_recipes()`:

```rust
CraftingRecipe {
    id: "my_recipe",
    name: "My Recipe",
    category: RecipeCategory::Weapon,
    materials: vec![
        CraftingMaterial { item_id: "scrap_metal", quantity: 3 },
        CraftingMaterial { item_id: "energy_core", quantity: 1 },
    ],
    result_item: "damage_amp",
    result_quantity: 1,
    craft_time: 4.0,
    required_level: 2,
},
```

Valid `item_id` values are the `id` fields in `all_items()` in [components/inventory.rs](src/components/inventory.rs).

### Adding a New Robot Preset

Open [robots/presets.rs](src/robots/presets.rs) and add a new `RobotStyle` entry in `robot_by_name()`. Then reference it by name in `spawn_enemy_entity()` or companion setup.

The `RobotStyle` fields control: scale, torso/head/arm/leg dimensions, wing span, cannons, shoulder pads, visor style, tail, antennae, colors (primary, secondary, emissive), and more.

### World Generation

The world is seeded and deterministic. The seed is set in `GameSettings::world_seed` (default `42_195`). Each zone gets `seed + offset` so they all generate independently.

To add a new zone:
1. Add a `WorldZone` variant in [components/world.rs](src/components/world.rs).
2. Write a `spawn_my_zone()` function in [world_plugin.rs](src/plugins/world_plugin.rs).
3. Call it from `generate_city()`.
4. Use `bevy_rapier3d::prelude::Collider` on any walkable geometry.

The `seeded(seed, index) -> f32` helper gives a deterministic `[0, 1)` float from two integers — use it for reproducible building positions, heights, and sizes.

### Damage Pipeline

```
Enemy attack
  → damage_player() in player_plugin.rs
      → parry check (blocks all damage if parrying)
      → ArmorSet::calculate_damage_reduction()
      → 70% absorbed by armor stat, 30% goes to health
      → apply_damage() in damage.rs
          → resistance_multiplier() (per-type resistances)
          → Health::apply_damage()
      → post-hit invulnerability (0.2s)
      → PlayerDamagedEvent fired
          → camera_shake_system reacts
          → damage_vignette_system reacts
          → save_plugin autosave timer resets
```

For enemy damage from player projectiles, the pipeline goes through `apply_damage()` directly in `projectile_update_system()`, then fires `EnemyDamagedEvent` → `EnemyKilledEvent`.

### Save System

Saves are written to `detroit3026_save.json` in the working directory. The save captures: player level, XP, credits, max health/stamina/armor, and wave number. It **does not** save inventory contents or world state (those reset on new session).

- **Auto-save**: every 30 seconds during play
- **Manual save**: `F5`
- **Load**: automatic on `OnEnter(AppState::Playing)`

To add new fields to the save, update `SaveData` in [save_plugin.rs](src/plugins/save_plugin.rs) and derive `Serialize`/`Deserialize` on anything new.

### Projectile Asset Cache

All projectile meshes and materials are pre-allocated once at startup in `setup_weapon_assets()` and stored in the `ProjectileAssets` resource. Fire systems read `Res<ProjectileAssets>` — they never call `meshes.add()` or `materials.add()` at runtime. This keeps fire-rate-heavy weapons (Laser: 20 shots/sec) allocation-free.

To add a new projectile visual:
1. Add handles to `ProjectileAssets`.
2. Populate them in `setup_weapon_assets()` using `mk_proj_mat()`.
3. Reference them by name in the relevant fire system.

---

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `bevy` | 0.15 | Game engine (ECS, rendering, input, windowing) |
| `bevy_rapier3d` | 0.28 | Physics (character controller, colliders, rigid bodies) |
| `rand` | 0.8 | RNG for enemy AI, loot, world generation |
| `noise` | 0.8 | Available for procedural generation (Perlin/Simplex) |
| `serde` + `serde_json` | 1.x | Save/load serialization |

### Feature Flags

```sh
# Dynamic linking — much faster incremental recompiles (dev only)
cargo run --features dynamic

# Release — LTO, single codegen unit, stripped binary (~30% smaller, 2× faster)
cargo build --release
```

---

## Game Systems Reference

### Wave Scaling

Each wave that completes increments the wave counter and applies:
- Enemy health/damage multiplier: `1.0 + (wave - 1) * 0.2`  
- Max enemy cap: `20 + 2 per wave` (capped at 50)
- Spawn interval: `5.0 - 0.2 per wave` (floor at 2.0 seconds)
- **Boss spawns** on waves 5, 10, 15… with `(2.0 + wave * 0.3)×` scaling and 3× health

### Player Stats (Default)

| Stat | Default | Notes |
|------|---------|-------|
| Max Health | 100 | +10 per level-up |
| Max Armor | 100 | Absorbs 70% of damage |
| Max Stamina | 100 | +5 per level-up; regens 10/sec |
| Jetpack Fuel | 200 | Regens when grounded |
| Dodge Cost | 20 stamina | 0.3s invulnerability, 0.5s cooldown |
| Level-up XP | `100 × level` | Fully heals on level-up |

### Enemy Types

| Type | HP | Damage | Speed | First appears |
|------|----|--------|-------|---------------|
| Drone | 50 | 8 | Fast | Wave 1 (35%) |
| Soldier | 100 | 15 | Medium | Wave 1 (30%) |
| Insectoid | 80 | 20 | Fast | Wave 1 (20%) |
| Heavy | 300 | 25 | Slow | Wave 3 (15%) |
| Hybrid | 1000 | 40 | Medium | Wave 5 (5%) |
| Boss | Hybrid×3 | Hybrid×scale | Medium | Every 5th wave |

### Primary Weapons

| Slot | Name | Damage | Fire Rate | Ammo | Notes |
|------|------|--------|-----------|------|-------|
| 1 | Plasma Pistol | 15 | 0.3s | 50 | Semi-auto |
| 2 | Pulse Rifle | 25 | 0.1s | 120 | Full-auto |
| 3 | Scatter Blaster | 8×8 | 0.8s | 24 | 8 pellets |
| 4 | Nova Launcher | 100 | 1.5s | 8 | Explosive r=6 |
| 5 | Photon Beam | 40 | 0.05s | 200 | Full-auto |
| 6 | Fusion Grenades | 80 | 1.0s | 6 | Arc + r=8 AoE |

### Beam Sabre Levels

| Level | Slash Dmg | Wave Dmg | Slashes | Cooldown | Bonus |
|-------|-----------|----------|---------|----------|-------|
| 1 | 25 | 40 | 2 | 0.8s | — |
| 2 | 35 | 60 | 2 | 0.7s | — |
| 3 | 50 | 80 | 3 | 0.6s | Piercing |
| 4 | 65 | 100 | 4 | 0.5s | Dual wave |
| 5 | 85 | 150 | 5 | 0.4s | AoE splash |

---

## Known Limitations / Roadmap

- **Audio**: no sound system yet — Bevy's audio plugin is available but no assets exist
- **Navmesh pathfinding**: enemy AI is distance-based; complex geometry can trap enemies
- **Inventory UI**: inventory exists but has no visual panel (crafting panel added; direct item inspection is `C`)
- **Armor equip UI**: armor pieces are code-set; no in-game equip screen yet
- **Level streaming**: world is fully loaded at once; very large maps may need chunking
- **Multiplayer**: single-player only
- **Animation**: robots are static meshes — no skeletal animation

---

## Contributing / Extending

This project follows a **plugin-per-system** convention. Each major gameplay area lives in its own plugin file under `src/plugins/`. When adding a major new feature:

1. Create `src/plugins/my_feature_plugin.rs`
2. Implement `pub struct MyFeaturePlugin` with `impl Plugin`
3. Export it from `src/plugins/mod.rs`
4. Register it in `main.rs`

Keep systems focused and communicate through events. Avoid querying components from other plugins' "domain" directly — fire an event instead.
