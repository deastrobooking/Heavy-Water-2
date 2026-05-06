use bevy::prelude::*;
use rand::Rng;

use crate::state::AppState;
use crate::events::*;
use crate::components::player::{Player, PlayerStats};
use crate::components::enemy::{Enemy, EnemyType, EnemyStateMachine, EnemyAIState, DeadEnemy, BossEnemy};
use crate::components::world::WorldLoot;
use crate::components::inventory::Inventory;
use crate::damage::{Health, Damageable, DamageInfo, DamageType, apply_damage};
use crate::resources::WaveInfo;
use crate::robots::presets::robot_by_name;
use crate::robots::factory::spawn_robot;

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WaveInfo>()
            .add_systems(OnEnter(AppState::Playing), setup_enemies)
            .add_systems(
                Update,
                (
                    wave_timer_system,
                    boss_spawn_system,
                    enemy_spawn_system,
                    enemy_ai_system,
                    enemy_attack_system,
                    enemy_dead_cleanup,
                    enemy_killed_reward,
                    enemy_loot_drop_system,
                    loot_bob_system,
                    loot_pickup_system,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

// ── Initial Setup ─────────────────────────────────────────────────────────────
fn setup_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_q: Query<&Transform, With<Player>>,
    mut wave: ResMut<WaveInfo>,
) {
    *wave = WaveInfo::new();
    let Ok(pt) = player_q.get_single() else { return };
    let player_pos = pt.translation;

    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let pos = random_spawn_pos(player_pos, &mut rng);
        spawn_enemy_entity(&mut commands, &mut meshes, &mut materials, EnemyType::Soldier, pos, 1.0);
    }
    wave.enemy_count = 5;
}

// ── Wave Timer ────────────────────────────────────────────────────────────────
fn wave_timer_system(
    time: Res<Time>,
    mut wave: ResMut<WaveInfo>,
    mut wave_ev: EventWriter<WaveStartedEvent>,
    mut completed_ev: EventWriter<WaveCompletedEvent>,
) {
    wave.wave_timer += time.delta_secs();
    wave.spawn_timer += time.delta_secs();

    if wave.wave_timer >= wave.wave_duration {
        wave.wave_timer = 0.0;
        completed_ev.send(WaveCompletedEvent);
        wave.advance();
        wave_ev.send(WaveStartedEvent { wave: wave.wave_number });
    }
}

// ── Boss Spawning (every 5th wave) ────────────────────────────────────────────
fn boss_spawn_system(
    mut wave_ev: EventReader<WaveStartedEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_q: Query<&Transform, With<Player>>,
    mut wave: ResMut<WaveInfo>,
    mut msg_ev: EventWriter<UiMessageEvent>,
    mut boss_ev: EventWriter<BossSpawnedEvent>,
) {
    for ev in wave_ev.read() {
        if ev.wave % 5 != 0 { continue; }

        let Ok(pt) = player_q.get_single() else { continue };
        // Spawn boss 30 units from player at a 45° angle
        let pos = pt.translation + Vec3::new(22.0, 0.0, 22.0);

        let boss_scale = 2.0 + ev.wave as f32 * 0.3;
        spawn_boss_entity(&mut commands, &mut meshes, &mut materials, pos, boss_scale);
        wave.enemy_count += 1;

        msg_ev.send(UiMessageEvent {
            text: format!("!! BOSS UNIT DETECTED - Wave {} !!", ev.wave),
            duration: 4.0,
        });
        boss_ev.send(BossSpawnedEvent { wave: ev.wave, position: pos });
    }
}

fn spawn_boss_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    scale: f32,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.0, 0.45),
        emissive: LinearRgba::new(1.5, 0.0, 2.5, 1.0),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..default()
    });

    let boss_enemy = Enemy::new(EnemyType::Hybrid, position, scale);
    let max_hp = boss_enemy.scaled_health() * 3.0;

    // Boss body: large distinctive shape
    let root = commands.spawn((
        PbrBundle {
            mesh: Mesh3d(meshes.add(Cuboid::new(3.0, 5.0, 3.0))),
            material: MeshMaterial3d(mat.clone()),
            transform: Transform::from_translation(position),
            ..default()
        },
        boss_enemy,
        EnemyStateMachine::default(),
        Health::new(max_hp),
        Damageable::default(),
        BossEnemy,
    )).id();

    // Glowing orb on top
    let orb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.0, 0.8),
        emissive: LinearRgba::new(3.0, 0.0, 5.0, 1.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: Mesh3d(meshes.add(Sphere::new(1.0))),
            material: MeshMaterial3d(orb_mat),
            transform: Transform::from_translation(position + Vec3::Y * 3.5),
            ..default()
        },
    ));
}

// ── Regular Enemy Spawning ────────────────────────────────────────────────────
fn enemy_spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wave: ResMut<WaveInfo>,
    player_q: Query<&Transform, With<Player>>,
    mut spawned_ev: EventWriter<EnemySpawnedEvent>,
) {
    if wave.enemy_count >= wave.max_enemies { return; }
    if wave.spawn_timer < wave.spawn_interval { return; }
    wave.spawn_timer = 0.0;

    let Ok(pt) = player_q.get_single() else { return };
    let player_pos = pt.translation;

    let mut rng = rand::thread_rng();
    let enemy_type = select_enemy_type(wave.wave_number, &mut rng);
    let pos = random_spawn_pos(player_pos, &mut rng);
    let scale = wave.difficulty_multiplier();

    spawn_enemy_entity(&mut commands, &mut meshes, &mut materials, enemy_type, pos, scale);
    wave.enemy_count += 1;

    spawned_ev.send(EnemySpawnedEvent {
        enemy_type: enemy_type.as_str().to_string(),
        position: pos,
    });
}

fn select_enemy_type(wave: u32, rng: &mut impl Rng) -> EnemyType {
    let roll: f32 = rng.gen();
    if wave >= 5 && roll < 0.05 { return EnemyType::Hybrid; }
    if wave >= 3 && roll < 0.15 { return EnemyType::Heavy; }
    if roll < 0.35 { return EnemyType::Drone; }
    if roll < 0.65 { return EnemyType::Soldier; }
    EnemyType::Insectoid
}

fn random_spawn_pos(player_pos: Vec3, rng: &mut impl Rng) -> Vec3 {
    let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
    let dist: f32 = rng.gen_range(30.0..80.0);
    Vec3::new(
        player_pos.x + angle.cos() * dist,
        player_pos.y,
        player_pos.z + angle.sin() * dist,
    )
}

fn spawn_enemy_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    enemy_type: EnemyType,
    position: Vec3,
    difficulty_scale: f32,
) {
    let preset_name = match enemy_type {
        EnemyType::Drone     => "JetWarden",
        EnemyType::Soldier   => "ScoutPrime",
        EnemyType::Heavy     => "TankTitan",
        EnemyType::Insectoid => "InsectoidStalker",
        EnemyType::Hybrid    => "HybridOmega",
    };

    let style = robot_by_name(preset_name).unwrap_or_default();
    let enemy_data = Enemy::new(enemy_type, position, difficulty_scale);
    let max_hp = enemy_data.scaled_health();

    let root = spawn_robot(commands, meshes, materials, &style, position);
    commands.entity(root).insert((
        enemy_data,
        EnemyStateMachine::default(),
        Health::new(max_hp),
        Damageable::default(),
    ));
}

// ── AI System ─────────────────────────────────────────────────────────────────
fn enemy_ai_system(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&mut Transform, &mut Enemy, &mut EnemyStateMachine, &Health), Without<Player>>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_q.get_single() else { return };
    let player_pos = player_transform.translation;
    let mut rng = rand::thread_rng();

    for (mut transform, mut enemy, mut sm, health) in enemy_q.iter_mut() {
        if !health.is_alive() { continue; }

        sm.timer += dt;
        enemy.attack_cooldown_timer = (enemy.attack_cooldown_timer - dt).max(0.0);

        let dist_to_player = transform.translation.distance(player_pos);
        let config = &enemy.config;

        match sm.current {
            EnemyAIState::Idle => {
                if dist_to_player < config.detection_range {
                    sm.transition(EnemyAIState::Chase);
                } else if sm.timer > rng.gen_range(1.0..3.0) {
                    let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                    let dist: f32 = rng.gen_range(10.0..20.0);
                    enemy.patrol_target = enemy.spawn_origin + Vec3::new(
                        angle.cos() * dist, 0.0, angle.sin() * dist,
                    );
                    sm.transition(EnemyAIState::Patrol);
                }
            }
            EnemyAIState::Patrol => {
                if dist_to_player < config.detection_range {
                    sm.transition(EnemyAIState::Chase);
                    continue;
                }
                let to_target = enemy.patrol_target - transform.translation;
                let to_target_flat = Vec3::new(to_target.x, 0.0, to_target.z);
                if to_target_flat.length() < 1.0 {
                    sm.transition(EnemyAIState::Idle);
                } else {
                    let move_dir = to_target_flat.normalize();
                    let pos = transform.translation;
                    transform.translation += move_dir * config.patrol_speed * dt * 60.0;
                    transform.look_at(pos + move_dir, Vec3::Y);
                }
            }
            EnemyAIState::Chase => {
                if dist_to_player > config.chase_range * 1.5 {
                    sm.transition(EnemyAIState::Patrol);
                } else if dist_to_player <= config.attack_range {
                    sm.transition(EnemyAIState::Attack);
                } else {
                    let to_player = (player_pos - transform.translation).with_y(0.0).normalize_or_zero();
                    transform.translation += to_player * config.chase_speed * dt * 60.0;
                    if to_player.length_squared() > 0.001 {
                        let pos = transform.translation;
                        transform.look_at(pos + to_player, Vec3::Y);
                    }
                    if enemy.enemy_type == EnemyType::Drone {
                        transform.translation.y = player_pos.y + 5.0
                            + (time.elapsed_secs() * 2.0 + transform.translation.x).sin() * 0.5;
                    }
                }
            }
            EnemyAIState::Attack => {
                if dist_to_player > config.attack_range * 1.3 {
                    sm.transition(EnemyAIState::Chase);
                } else {
                    let to_player = (player_pos - transform.translation).with_y(0.0).normalize_or_zero();
                    if to_player.length_squared() > 0.001 {
                        let pos = transform.translation;
                        transform.look_at(pos + to_player, Vec3::Y);
                    }
                }
            }
            EnemyAIState::Stunned => {
                if sm.timer >= 1.5 { sm.transition(EnemyAIState::Chase); }
            }
            EnemyAIState::Dead => {}
        }
    }
}

// ── Attack System ─────────────────────────────────────────────────────────────
fn enemy_attack_system(
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&Transform, &mut Enemy, &EnemyStateMachine, &Health), Without<Player>>,
    mut player_damage_q: Query<
        (&mut crate::damage::Health, &mut Damageable, &mut PlayerStats, &mut crate::components::player::ParryState, &crate::components::armor::ArmorSet),
        With<Player>,
    >,
    mut damaged_ev: EventWriter<PlayerDamagedEvent>,
    mut parry_ev: EventWriter<PlayerParryEvent>,
) {
    let Ok(player_transform) = player_q.get_single() else { return };
    let player_pos = player_transform.translation;
    let mut total_damage = 0.0;

    for (e_transform, mut enemy, sm, health) in enemy_q.iter_mut() {
        if !health.is_alive() { continue; }
        if sm.current != EnemyAIState::Attack { continue; }
        if enemy.attack_cooldown_timer > 0.0 { continue; }
        let dist = e_transform.translation.distance(player_pos);
        if dist <= enemy.config.attack_range {
            total_damage += enemy.scaled_damage();
            enemy.attack_cooldown_timer = enemy.config.attack_cooldown;
        }
    }

    if total_damage > 0.0 {
        if let Ok((mut health, mut damageable, mut stats, mut parry, armor)) = player_damage_q.get_single_mut() {
            crate::plugins::player_plugin::damage_player(
                &mut health, &mut damageable, &mut stats, &mut parry, &armor,
                &DamageInfo::new(total_damage, DamageType::Kinetic),
                &mut damaged_ev, &mut parry_ev,
            );
        }
    }
}

// ── Dead Cleanup ──────────────────────────────────────────────────────────────
fn enemy_dead_cleanup(
    mut commands: Commands,
    time: Res<Time>,
    mut dead_q: Query<(Entity, &mut DeadEnemy)>,
    mut wave: ResMut<WaveInfo>,
) {
    let dt = time.delta_secs();
    for (entity, mut dead) in dead_q.iter_mut() {
        dead.despawn_timer -= dt;
        if dead.despawn_timer <= 0.0 {
            commands.entity(entity).despawn_recursive();
            wave.enemy_count = wave.enemy_count.saturating_sub(1);
        }
    }
}

// ── Rewards on Kill ───────────────────────────────────────────────────────────
fn enemy_killed_reward(
    mut killed_ev: EventReader<EnemyKilledEvent>,
    mut player_q: Query<&mut PlayerStats, With<Player>>,
    mut enemy_q: Query<(Entity, &mut EnemyStateMachine, &Health), Without<Player>>,
    mut commands: Commands,
) {
    let Ok(mut stats) = player_q.get_single_mut() else { return };
    for ev in killed_ev.read() {
        stats.credits += ev.credits;
        stats.experience += ev.experience;
    }
    for (entity, mut sm, health) in enemy_q.iter_mut() {
        if !health.is_alive() && sm.current != EnemyAIState::Dead {
            sm.force(EnemyAIState::Dead);
            commands.entity(entity).insert(DeadEnemy { despawn_timer: 2.0 });
        }
    }
}

// ── Loot Drop on Kill ─────────────────────────────────────────────────────────
fn enemy_loot_drop_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut killed_ev: EventReader<EnemyKilledEvent>,
) {
    let mut rng = rand::thread_rng();

    for ev in killed_ev.read() {
        // ~60% drop chance
        let roll: f32 = rng.gen();
        if roll > 0.60 { continue; }

        let (item_id, quantity, r, g, b): (&'static str, u32, f32, f32, f32) = if roll < 0.10 {
            ("health_pack",    1, 0.2, 1.0, 0.3)
        } else if roll < 0.20 {
            ("armor_shard",    1, 0.3, 0.5, 1.0)
        } else if roll < 0.35 {
            ("plasma_cell",    rng.gen_range(10..25), 0.0, 0.6, 1.0)
        } else if roll < 0.48 {
            ("scrap_metal",    rng.gen_range(1..4),   0.6, 0.5, 0.3)
        } else {
            ("energy_core",    1, 1.0, 0.8, 0.0)
        };

        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(r, g, b),
            emissive: LinearRgba::new(r * 1.5, g * 1.5, b * 1.5, 1.0),
            unlit: false,
            metallic: 0.5,
            ..default()
        });

        let base_y = ev.position.y + 0.6;
        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(meshes.add(Sphere::new(0.35))),
                material: MeshMaterial3d(mat),
                transform: Transform::from_translation(Vec3::new(ev.position.x, base_y, ev.position.z)),
                ..default()
            },
            WorldLoot { item_id, quantity, credits: 0, pickup_radius: 2.5, base_y },
        ));
    }
}

// ── Loot Bob Animation ────────────────────────────────────────────────────────
fn loot_bob_system(
    time: Res<Time>,
    mut loot_q: Query<(&mut Transform, &WorldLoot)>,
) {
    for (mut transform, loot) in loot_q.iter_mut() {
        transform.translation.y = loot.base_y + (time.elapsed_secs() * 2.5).sin() * 0.25;
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs());
    }
}

// ── Loot Pickup ───────────────────────────────────────────────────────────────
fn loot_pickup_system(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    mut inventory_q: Query<&mut Inventory, With<Player>>,
    loot_q: Query<(Entity, &Transform, &WorldLoot)>,
    mut msg_ev: EventWriter<UiMessageEvent>,
    mut loot_ev: EventWriter<LootCollectedEvent>,
) {
    let Ok(player_transform) = player_q.get_single() else { return };
    let Ok(mut inventory) = inventory_q.get_single_mut() else { return };
    let player_pos = player_transform.translation;
    let item_defs = crate::components::inventory::all_items();

    for (entity, loot_transform, loot) in loot_q.iter() {
        let dist = player_pos.distance(loot_transform.translation);
        if dist > loot.pickup_radius { continue; }

        let max_stack = item_defs.iter()
            .find(|i| i.id == loot.item_id)
            .map(|i| i.max_stack)
            .unwrap_or(10);

        let leftover = inventory.add_item(loot.item_id, loot.quantity, max_stack);
        let picked = loot.quantity.saturating_sub(leftover);
        if picked > 0 {
            msg_ev.send(UiMessageEvent {
                text: format!("Picked up {}x {}", picked, loot.item_id.replace('_', " ")),
                duration: 1.8,
            });
            loot_ev.send(LootCollectedEvent {
                loot_type: loot.item_id.to_string(),
                amount: picked,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}
