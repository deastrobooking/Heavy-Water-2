# Heavy Water

A 3D open-world sci-fi action RPG built with [Bevy 0.15](https://bevyengine.org/) and Rust.
Hand-authored 14-chapter campaign across the lives of four robotic factions —
**Synthetic**, **Mechanoid**, **Insectoid**, **Swarm**, plus the **Animatons** and the
**Charred** — with chassis customization, recruitable companions, weapon/armor mods,
diegetic radio chatter narrative, and discoverable easter-egg upgrades on every map.

You play **Amp**, the firstborn Synthetic, awakening into a ruined world and gathering
your scattered brothers and sisters across hostile biomes ranging from the Insectoid
Metropolis to Char's Domain to the Starfall Arena.

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

---

## Controls

All input flows through the unified `GameInput` resource (`src/plugins/input_plugin.rs`).
Both keyboard/mouse and any connected gamepad (USB wired or Bluetooth wireless) are
supported simultaneously — the system picks the active gamepad automatically.

### Keyboard & Mouse

| Key / Button | Action |
|---|---|
| `WASD` | Move |
| Mouse | Look (cursor locked) |
| `Shift` | Sprint (drains stamina) |
| `Space` | Jump / hold for Jetpack |
| `LMB` | Fire active weapon / Beam Sabre slash |
| `RMB` | Aim down sights |
| `1–6` | Select primary weapon slot |
| `[ ]` | Cycle weapon slot backward / forward |
| `7` | Homing Missile |
| `8` | Energy Burst (3-bolt pierce) |
| `9` | Bomb (gravity arc, 12u AoE) |
| `0` | Combat Drone burst |
| `R` | Reload |
| `V` | Light melee combo (Jab → Cross → Uppercut) |
| `B` | Heavy melee combo (Slam → Sweep) |
| `T` | Toggle Beam Sabre (must be unlocked in Ch.1) |
| `Q` | Dodge roll (costs stamina, grants invulnerability) |
| `F` | Parry (0.2s window, blocks all damage) |
| `E` | Interact |
| `C` | Toggle crafting panel |
| `M` | Motorcycle (requires blueprint, Ch.1) |
| `J` | Jet (requires blueprint, Ch.2) |
| `F5` | Manual save |
| `Esc` | Pause / return to main menu |

### Controller (Xbox / PlayStation / Generic HID)

PlayStation button labels shown in parentheses.

| Button | Action |
|---|---|
| Left stick | Move |
| Right stick | Look (quadratic sensitivity curve) |
| South — A / ✕ | Jump / hold for Jetpack |
| East  — B / ○ | Dodge roll |
| West  — X / □ | Reload |
| North — Y / △ | Parry |
| RT / R2 | Fire |
| LT / L2 | Aim down sights |
| LB / L1 | Sprint |
| RB / R1 | Next weapon |
| D-Pad Right | Next weapon |
| D-Pad Left | Previous weapon |
| D-Pad Up | Enter vehicle / Jet |
| D-Pad Down | Interact |
| L3 (left stick click) | Heavy melee |
| R3 (right stick click) | Light melee |
| Select / Share | Crafting panel |
| Start / Options | Pause / menu |
| Mode / Guide | Beam Sabre toggle |

---

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

---

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
    ├── resources.rs            # WaveInfo, GameSettings, PlayerScore, CameraShake
    ├── components/
    │   ├── player.rs           # PlayerStats, PlayerMovement, JetpackState, DodgeState, ParryState
    │   ├── enemy.rs            # Enemy, EnemyConfig, EnemyStateMachine, BossEnemy
    │   ├── weapon.rs           # Weapon, WeaponInventory, SpecialWeapon, BeamSabre, Projectile, MeleeCombo
    │   ├── armor.rs            # ArmorSet, ArmorPiece, ElementType
    │   ├── inventory.rs        # Inventory, ItemDefinition, all_items()
    │   ├── companion.rs        # Companion, CompanionKind, CompanionProjectile
    │   └── world.rs            # WorldGeometry, Building, Chest, WorldLoot, NeonLight
    ├── lsystem/
    │   ├── mod.rs              # LSystem string rewriter (axiom + production rules, N iterations)
    │   ├── turtle.rs           # 3-D turtle graphics interpreter → BranchSegment / LeafCluster
    │   └── tree.rs             # TreeKind enum, TreeTemplate cache, spawn_tree()
    ├── plugins/
    │   ├── mod.rs
    │   ├── input_plugin.rs     # Unified GameInput resource — keyboard/mouse + gamepad (USB & BT)
    │   ├── player_plugin.rs    # Movement, dodge, parry, jetpack, camera shake, level-up
    │   ├── weapon_plugin.rs    # Fire systems, special weapons, beam sabre, melee, VFX, particles
    │   ├── enemy_plugin.rs     # Wave timer, spawning, AI, loot drops, boss encounters
    │   ├── world_plugin.rs     # Procedural world: terrain mesh, 8 zone types, trees, palette
    │   ├── chest_plugin.rs     # Loot chest interaction
    │   ├── companion_plugin.rs # Companion follow, combat, healing
    │   ├── armor_plugin.rs     # Armor stat bonuses, elemental switching
    │   ├── crafting_plugin.rs  # Recipe system, craft queue
    │   ├── ui_plugin.rs        # HUD, menus, damage vignette, crafting panel
    │   ├── save_plugin.rs      # JSON checkpoint save/load (F5 / autosave every 30s)
    │   ├── chapter_plugin.rs   # Chapter script runner, encounter sequences
    │   ├── discoverable_plugin.rs # Hidden collectibles, blueprints, perks
    │   ├── radio_plugin.rs     # Diegetic radio narrative system
    │   ├── vehicle_plugin.rs   # Motorcycle / jet buff toggles
    │   └── chassis_editor_plugin.rs # In-game robot chassis customization
    └── robots/
        ├── designer.rs         # RobotStyle — 20+ procedural parameters
        ├── factory.rs          # Spawns robot mesh hierarchy from RobotStyle
        └── presets.rs          # 12 named robot presets (ScoutPrime, TankTitan, HybridOmega…)
```

---

## Developer Guide

### Architecture Overview

The game uses Bevy's **ECS (Entity Component System)** throughout. Three key patterns:

1. **State-gated systems** — every gameplay system runs inside `.run_if(in_state(AppState::Playing))` so it only ticks during active play.
2. **Event-driven communication** — systems never query each other directly. They fire events (`EnemyKilledEvent`, `PlayerDamagedEvent`, etc.) that other systems react to. All events are declared in [events.rs](src/events.rs) and registered in `EventsPlugin`.
3. **Component composition** — entities are built from small, single-purpose components. The player entity has ~12 components stapled together in `spawn_player()`.

### Input System

All input is centralised in `GameInput` (`src/plugins/input_plugin.rs`). The `update_game_input` system runs in `PreUpdate` and populates `GameInput` from keyboard, mouse, and the first connected gamepad each frame. Game systems then read `Res<GameInput>` instead of raw device resources.

```rust
// Reading input in any system:
fn my_system(gi: Res<GameInput>) {
    if gi.fire { /* fire weapon */ }
    let dir = gi.move_axis; // Vec2, normalised
    let look = gi.look_delta; // Vec2, radians this frame
}
```

To add a new action:
1. Add a `bool` (or other) field to `GameInput` in `input_plugin.rs`.
2. Populate it from keyboard/gamepad in `update_game_input`.
3. Read it in the relevant system.

### World Generation

The world is seeded and fully deterministic. The seed lives in `GameSettings::world_seed` (default `42_195`). Each zone receives `seed + offset` so zones generate independently.

#### Terrain

A 160×160 quad mesh (25,600 triangles) covers the full 1 200 × 1 200 unit world. Heights are computed from layered sine-wave octaves plus ridge lines and corner-peak boosts. A smoothstep mask flattens the city centre (within ~120 u of origin) so buildings sit level. A `Collider::trimesh` provides physics.

```rust
// Height function signature (world_plugin.rs):
fn terrain_height(x: f32, z: f32, seed: u64) -> f32
```

#### Material Palette

All world geometry shares a `Palette` struct holding 14 `Handle<StandardMaterial>` — one per visual theme. Zone spawn functions receive `&Palette` and `.clone()` handles rather than allocating a new GPU material per mesh. To add a new material, add a field to `Palette` and populate it in `Palette::build()`.

#### L-System Trees

Trees are generated with a formal string-rewriting L-system evaluated once per species into a `TurtleResult` (branch segments + leaf clusters), then cached in `TreeTemplate`. `spawn_tree()` clones handles from the template, keeping GPU allocations at 2 per species regardless of instance count.

| Species | Grammar | Iterations | Description |
|---------|---------|-----------|-------------|
| Oak | `F → F[+F][-F]` | 4 | Spreading deciduous, green leaves |
| Pine | `F → F[^F][-F][+F]` | 3 | Narrow conifer, dark green |
| Dead | `F → F[+F]F[-F]` | 3 | Bare skeleton, no leaves |
| Cyber | `F → F[^F][-F][+F]` | 3 | Neon emissive fantasy tree |

48 trees are scattered across 6 zone-specific placement groups using seeded positions.

#### Zones

| Zone | Approx. location | Visual theme |
|------|-----------------|-------------|
| Downtown | Centre (±100 u) | Glass curtain-wall towers, cyan/orange/green glow |
| Industrial | East (+120 → +320 u) | Corrugated metal, rust, heat-stained concrete |
| Residential | West (−120 → −320 u) | Weathered painted concrete |
| Highways | Cross-shaped elevated roads | Dark asphalt, metal pillars |
| Sky Platforms | Random, 40–250 u altitude | Blue-chrome discs |
| Sky Bridges | Random, 60–160 u altitude | Blue-chrome spans |
| Spaceports | Four corners (±350 u) | Circular landing pads |
| Mountains | Corner clusters (±460 u) | Layered rock spires + snow caps |
| River | Sinusoidal through centre | Semi-transparent water |
| Outer Districts | Cardinal + diagonal (300 u) | Low-rise residential blocks |
| Trees | Zone-contextual | Oak / Pine / Dead / Cyber |

To add a new zone:
1. Add a `WorldZone` variant in [components/world.rs](src/components/world.rs).
2. Write a `spawn_my_zone(commands, meshes, pal, seed)` function in [world_plugin.rs](src/plugins/world_plugin.rs).
3. Call it from `generate_city()`.
4. Tag walkable geometry with `WalkableSurface` and a `Collider`.

The `seeded(seed, index) -> f32` helper returns a deterministic `[0, 1)` float — use it for reproducible positions, heights, and sizes.

### Adding a New System

1. Write the system function in the appropriate plugin file.
2. Add it to the plugin's `.add_systems(Update, your_system.run_if(in_state(AppState::Playing)))` call.
3. Use `.chain()` / `.before()` / `.after()` for ordering.

### Adding a New Event

1. Declare the struct in [events.rs](src/events.rs):
```rust
#[derive(Event, Debug)]
pub struct MyNewEvent { pub value: f32 }
```
2. Register it in `EventsPlugin::build()`: `.add_event::<MyNewEvent>()`
3. Fire with `EventWriter<MyNewEvent>`, read with `EventReader<MyNewEvent>`.

### Adding a New Enemy Type

1. Add the variant to `EnemyType` in [components/enemy.rs](src/components/enemy.rs).
2. Add its config in `EnemyConfig::for_type()`.
3. Add a spawn weight in `select_enemy_type()` in [enemy_plugin.rs](src/plugins/enemy_plugin.rs).
4. Map it to a robot preset in `spawn_enemy_entity()` or create a new preset in [robots/presets.rs](src/robots/presets.rs).

### Adding a New Weapon

**Primary (slots 1–6):**
1. Add the variant to `WeaponType` in [components/weapon.rs](src/components/weapon.rs).
2. Add its stats in `Weapon::new()` (damage, fire rate, ammo, pellets, spread, etc.).
3. Add it to `WeaponInventory::default()`.
4. Map it to a mesh/material in `weapon_fire_system()`.
5. Add a slot label to the HUD weapon bar in [ui_plugin.rs](src/plugins/ui_plugin.rs).

**Special (slots 7–0):** Add a branch in `special_weapon_system()`. Special weapons reuse the `Projectile` component — configure damage, speed, explosion radius, and mesh size.

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
      → post-hit invulnerability (0.2 s)
      → PlayerDamagedEvent fired
          → camera_shake_system reacts
          → damage_vignette_system reacts
          → save_plugin autosave timer resets
```

Player projectile hits go through `apply_damage()` in `projectile_update_system()`, then fire `EnemyDamagedEvent` → `EnemyKilledEvent`.

### Save System

Saves are written to `detroit3026_save.json` in the working directory. The save captures: player level, XP, credits, max health/stamina/armor, and wave number. It **does not** save inventory contents or world state (those reset on new session).

- **Auto-save**: every 30 seconds during play
- **Manual save**: `F5`
- **Load**: automatic on `OnEnter(AppState::Playing)`

To add new fields update `SaveData` in [save_plugin.rs](src/plugins/save_plugin.rs) and derive `Serialize`/`Deserialize` on anything new.

### Projectile Asset Cache

All projectile meshes and materials are pre-allocated once at startup in `setup_weapon_assets()` and stored in the `ProjectileAssets` resource. Fire systems never call `meshes.add()` or `materials.add()` at runtime — keeping high-fire-rate weapons (Laser: 20 shots/sec) allocation-free.

---

## Game Systems Reference

### Wave Scaling

| Stat | Formula |
|------|---------|
| HP / damage multiplier | `1.0 + (wave − 1) × 0.2` |
| Max enemies | `20 + 2 × wave` (cap 50) |
| Spawn interval | `5.0 − 0.2 × wave` (floor 2.0 s) |
| Boss | Every 5th wave — `(2.0 + wave × 0.3)×` scaling, 3× HP |

### Player Stats (Default)

| Stat | Default | Notes |
|------|---------|-------|
| Max Health | 100 | +10 per level-up |
| Max Armor | 100 | Absorbs 70% of damage |
| Max Stamina | 100 | +5 per level-up; regens 10/sec |
| Jetpack Fuel | 200 | Regens when grounded |
| Dodge Cost | 20 stamina | 0.3 s invulnerability, 0.5 s cooldown |
| Level-up XP | `100 × level` | Full heal on level-up |

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
| 1 | Plasma Pistol | 15 | 0.3 s | 50 | Semi-auto |
| 2 | Pulse Rifle | 25 | 0.1 s | 120 | Full-auto |
| 3 | Scatter Blaster | 8×8 | 0.8 s | 24 | 8 pellets |
| 4 | Nova Launcher | 100 | 1.5 s | 8 | Explosive r=6 |
| 5 | Photon Beam | 40 | 0.05 s | 200 | Full-auto |
| 6 | Fusion Grenades | 80 | 1.0 s | 6 | Arc + r=8 AoE |

### Beam Sabre Levels

| Level | Slash Dmg | Wave Dmg | Slashes | Cooldown | Bonus |
|-------|-----------|----------|---------|----------|-------|
| 1 | 25 | 40 | 2 | 0.8 s | — |
| 2 | 35 | 60 | 2 | 0.7 s | — |
| 3 | 50 | 80 | 3 | 0.6 s | Piercing |
| 4 | 65 | 100 | 4 | 0.5 s | Dual wave |
| 5 | 85 | 150 | 5 | 0.4 s | AoE splash |

---

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `bevy` | 0.15 | Game engine (ECS, rendering, input, windowing) |
| `bevy_rapier3d` | 0.28 | Physics (character controller, colliders, rigid bodies) |
| `rand` | 0.8 | RNG for enemy AI, loot, world generation |
| `serde` + `serde_json` | 1.x | Save/load serialization |

### Feature Flags

```sh
# Dynamic linking — much faster incremental recompiles (dev only)
cargo run --features dynamic

# Release — LTO, single codegen unit, stripped binary (~30% smaller, 2× faster)
cargo build --release
```

---

## Known Limitations / Roadmap

- **Audio**: no sound system yet — Bevy's audio plugin is available but no assets exist
- **Navmesh pathfinding**: enemy AI is distance-based; complex geometry can trap enemies
- **Inventory UI**: crafting panel exists; direct item inspection not yet implemented
- **Armor equip UI**: armor pieces are code-set; no in-game equip screen yet
- **Level streaming**: world is fully loaded at once; very large maps may need chunking
- **Multiplayer**: single-player only
- **Animation**: robots are static meshes — no skeletal animation
- **Controller remapping**: bindings are hard-coded in `input_plugin.rs`; a settings screen with serialised remapping is planned

---

## Contributing / Extending

This project follows a **plugin-per-system** convention. Each major gameplay area lives in its own plugin file under `src/plugins/`. When adding a major new feature:

1. Create `src/plugins/my_feature_plugin.rs`
2. Implement `pub struct MyFeaturePlugin` with `impl Plugin`
3. Export it from `src/plugins/mod.rs`
4. Register it in `main.rs`

Keep systems focused and communicate through events. Avoid querying components from another plugin's domain directly — fire an event instead.
