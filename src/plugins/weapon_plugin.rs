use bevy::prelude::*;

use crate::state::AppState;
use crate::events::*;
use crate::components::player::*;
use crate::components::weapon::*;
use crate::components::enemy::Enemy;
use crate::damage::{Health, Damageable, DamageInfo, DamageType, apply_damage, area_damage_falloff};
use crate::plugins::input_plugin::GameInput;

// ── Hit Particle ──────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct HitParticle {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec3,
}

// ── Projectile Asset Cache ────────────────────────────────────────────────────
// Initialized once at startup so fire systems never allocate new GPU assets.
#[derive(Resource)]
pub struct ProjectileAssets {
    pub sphere_sm: Handle<Mesh>,
    pub sphere_md: Handle<Mesh>,
    pub sphere_lg: Handle<Mesh>,
    pub flash_sphere: Handle<Mesh>,
    pub mat_pistol: Handle<StandardMaterial>,
    pub mat_rifle: Handle<StandardMaterial>,
    pub mat_shotgun: Handle<StandardMaterial>,
    pub mat_rocket: Handle<StandardMaterial>,
    pub mat_laser: Handle<StandardMaterial>,
    pub mat_grenade: Handle<StandardMaterial>,
    pub mat_missile: Handle<StandardMaterial>,
    pub mat_energy: Handle<StandardMaterial>,
    pub mat_bomb: Handle<StandardMaterial>,
    pub mat_drone_shot: Handle<StandardMaterial>,
    pub mat_companion: Handle<StandardMaterial>,
    pub mat_melee_flash: Handle<StandardMaterial>,
    pub mat_hit_particle: Handle<StandardMaterial>,
}

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_weapon_assets)
            .add_systems(
                Update,
                (
                    weapon_select_system,
                    weapon_fire_system,
                    weapon_reload_system,
                    special_weapon_system,
                    projectile_update_system,
                    melee_combo_system,
                    beam_sabre_update_system,
                    hit_particle_spawn_system,
                    particle_update_system,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn mk_proj_mat(
    materials: &mut Assets<StandardMaterial>,
    r: f32, g: f32, b: f32,
    er: f32, eg: f32, eb: f32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(r, g, b),
        emissive: LinearRgba::new(er, eg, eb, 1.0),
        unlit: true,
        ..default()
    })
}

fn setup_weapon_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let m = &mut *materials;
    commands.insert_resource(ProjectileAssets {
        sphere_sm:        meshes.add(Sphere::new(0.08)),
        sphere_md:        meshes.add(Sphere::new(0.22)),
        sphere_lg:        meshes.add(Sphere::new(0.42)),
        flash_sphere:     meshes.add(Sphere::new(0.9)),
        mat_pistol:       mk_proj_mat(m, 0.0, 0.8, 1.0,  0.0, 2.0, 4.0),
        mat_rifle:        mk_proj_mat(m, 0.8, 1.0, 0.2,  1.5, 3.0, 0.0),
        mat_shotgun:      mk_proj_mat(m, 1.0, 0.5, 0.0,  3.0, 1.0, 0.0),
        mat_rocket:       mk_proj_mat(m, 1.0, 0.2, 0.0,  4.0, 0.5, 0.0),
        mat_laser:        mk_proj_mat(m, 0.8, 0.9, 1.0,  2.0, 3.0, 5.0),
        mat_grenade:      mk_proj_mat(m, 0.2, 1.0, 0.3,  0.0, 3.0, 0.5),
        mat_missile:      mk_proj_mat(m, 1.0, 0.4, 0.0,  3.0, 1.0, 0.0),
        mat_energy:       mk_proj_mat(m, 0.5, 0.0, 1.0,  1.5, 0.0, 4.0),
        mat_bomb:         mk_proj_mat(m, 0.8, 0.0, 0.6,  2.5, 0.0, 2.0),
        mat_drone_shot:   mk_proj_mat(m, 1.0, 0.8, 0.0,  3.0, 2.0, 0.0),
        mat_companion:    mk_proj_mat(m, 0.0, 1.0, 0.5,  0.0, 3.0, 1.5),
        mat_melee_flash:  mk_proj_mat(m, 1.0, 0.9, 0.3,  4.0, 3.0, 0.5),
        mat_hit_particle: mk_proj_mat(m, 1.0, 0.5, 0.0,  3.0, 1.0, 0.0),
    });
}

// ── Weapon Select ─────────────────────────────────────────────────────────────
fn weapon_select_system(
    gi:       Res<GameInput>,
    mut player_q: Query<&mut WeaponInventory, With<Player>>,
    mut switched_ev: EventWriter<WeaponSwitchedEvent>,
) {
    let Ok(mut inv) = player_q.get_single_mut() else { return };
    let prev  = inv.active_slot;
    let count = inv.slots.len();

    // Direct slot via number keys.
    let mut new_slot = gi.weapon_slot;

    // Cycle forward / backward via bumpers or bracket keys.
    if gi.weapon_next {
        new_slot = Some((prev + 1) % count);
    } else if gi.weapon_prev && prev > 0 {
        new_slot = Some(prev - 1);
    } else if gi.weapon_prev && prev == 0 {
        new_slot = Some(count - 1);
    }

    if let Some(s) = new_slot {
        if s < count {
            inv.active_slot = s;
            if s != prev {
                switched_ev.send(WeaponSwitchedEvent {
                    weapon_name: inv.active().weapon_type.display_name().to_string(),
                });
            }
        }
    }
}

// ── Primary Weapon Fire ───────────────────────────────────────────────────────
fn weapon_fire_system(
    gi:   Res<GameInput>,
    time: Res<Time>,
    mut commands: Commands,
    proj_assets: Res<ProjectileAssets>,
    mut player_q: Query<(&mut WeaponInventory, &mut PlayerStateMachine), With<Player>>,
    cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut fired_ev: EventWriter<WeaponFiredEvent>,
) {
    let dt = time.delta_secs();
    let Ok((mut inv, mut sm)) = player_q.get_single_mut() else { return };
    let Ok(cam) = cam_q.get_single() else { return };

    let firing     = gi.fire;
    let just_fired = gi.fire_just;

    // Don't fire primary weapons while beam sabre is active
    // (beam sabre gets LMB priority when active)
    let weapon = inv.active_mut();
    weapon.fire_timer = (weapon.fire_timer - dt).max(0.0);

    let should_fire = if weapon.automatic { firing } else { just_fired };
    if !should_fire || !weapon.can_fire() { return; }

    let pos = cam.translation();
    let forward = cam.forward().as_vec3();
    let right = cam.right().as_vec3();
    let up = cam.up().as_vec3();

    let damage = weapon.damage;
    let speed = weapon.speed;
    let spread = weapon.spread;
    let pellets = weapon.pellets;
    let is_explosive = weapon.is_explosive;
    let explosion_radius = weapon.explosion_radius;
    let gravity_affected = weapon.weapon_type == WeaponType::Grenade;

    let (mesh_h, mat_h) = match weapon.weapon_type {
        WeaponType::Pistol  => (proj_assets.sphere_sm.clone(), proj_assets.mat_pistol.clone()),
        WeaponType::Rifle   => (proj_assets.sphere_sm.clone(), proj_assets.mat_rifle.clone()),
        WeaponType::Shotgun => (proj_assets.sphere_sm.clone(), proj_assets.mat_shotgun.clone()),
        WeaponType::Rocket  => (proj_assets.sphere_md.clone(), proj_assets.mat_rocket.clone()),
        WeaponType::Laser   => (proj_assets.sphere_sm.clone(), proj_assets.mat_laser.clone()),
        WeaponType::Grenade => (proj_assets.sphere_md.clone(), proj_assets.mat_grenade.clone()),
    };

    weapon.ammo = weapon.ammo.saturating_sub(1);
    weapon.fire_timer = weapon.fire_rate;

    for _ in 0..pellets {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let (sx, sy) = if spread > 0.0 {
            (rng.gen_range(-spread..spread), rng.gen_range(-spread..spread))
        } else {
            (0.0, 0.0)
        };
        let dir = (forward + right * sx + up * sy).normalize();

        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(mesh_h.clone()),
                material: MeshMaterial3d(mat_h.clone()),
                transform: Transform::from_translation(pos + forward * 0.5),
                ..default()
            },
            Projectile {
                damage,
                speed,
                direction: dir,
                lifetime: 3.0,
                is_explosive,
                explosion_radius,
                weapon_type: ProjectileOwner::Player,
                owner: None,
                piercing: false,
                gravity_affected,
                vertical_velocity: if gravity_affected { 0.2 } else { 0.0 },
            },
        ));
    }

    sm.transition(PlayerState::Attacking);
    fired_ev.send(WeaponFiredEvent);
}

// ── Reload ────────────────────────────────────────────────────────────────────
fn weapon_reload_system(
    gi:       Res<GameInput>,
    mut player_q: Query<&mut WeaponInventory, With<Player>>,
    mut reload_ev: EventWriter<WeaponReloadedEvent>,
) {
    if gi.reload {
        if let Ok(mut inv) = player_q.get_single_mut() {
            inv.active_mut().reload();
            reload_ev.send(WeaponReloadedEvent);
        }
    }
}

// ── Special Weapons (7/8/9/0) ────────────────────────────────────────────────
fn special_weapon_system(
    gi:       Res<GameInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    proj_assets: Res<ProjectileAssets>,
    mut player_q: Query<&mut SpecialWeaponInventory, With<Player>>,
    cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut fired_ev: EventWriter<WeaponFiredEvent>,
    mut msg_ev: EventWriter<UiMessageEvent>,
) {
    let dt = time.delta_secs();
    let Ok(mut inv) = player_q.get_single_mut() else { return };
    let Ok(cam) = cam_q.get_single() else { return };

    // Tick all cooldowns every frame
    inv.slot7.cooldown_timer = (inv.slot7.cooldown_timer - dt).max(0.0);
    inv.slot8.cooldown_timer = (inv.slot8.cooldown_timer - dt).max(0.0);
    inv.slot9.cooldown_timer = (inv.slot9.cooldown_timer - dt).max(0.0);
    inv.slot0.cooldown_timer = (inv.slot0.cooldown_timer - dt).max(0.0);

    let pos = cam.translation();
    let fwd = cam.forward().as_vec3();

    // Slot 7 – Homing Missile
    if keyboard.just_pressed(KeyCode::Digit7) {
        if inv.slot7.can_fire() {
            let dmg = inv.slot7.effective_damage();
            inv.slot7.cooldown_timer = inv.slot7.cooldown;
            inv.slot7.ammo = inv.slot7.ammo.saturating_sub(1);
            commands.spawn((
                PbrBundle {
                    mesh: Mesh3d(proj_assets.sphere_md.clone()),
                    material: MeshMaterial3d(proj_assets.mat_missile.clone()),
                    transform: Transform::from_translation(pos + fwd),
                    ..default()
                },
                Projectile {
                    damage: dmg, speed: 35.0, direction: fwd,
                    lifetime: 5.0, is_explosive: true, explosion_radius: 5.0,
                    weapon_type: ProjectileOwner::Missile,
                    owner: None, piercing: false, gravity_affected: false, vertical_velocity: 0.0,
                },
            ));
            fired_ev.send(WeaponFiredEvent);
            msg_ev.send(UiMessageEvent {
                text: format!("Homing Missile! [{} ammo]", inv.slot7.ammo), duration: 1.5,
            });
        } else {
            msg_ev.send(UiMessageEvent { text: "Missile cooling down!".to_string(), duration: 1.0 });
        }
    }

    // Slot 8 – Energy Burst
    if keyboard.just_pressed(KeyCode::Digit8) {
        if inv.slot8.can_fire() {
            let dmg = inv.slot8.effective_damage();
            inv.slot8.cooldown_timer = inv.slot8.cooldown;
            inv.slot8.ammo = inv.slot8.ammo.saturating_sub(1);
            // Spawn 3 spread bolts in a tight cone
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let right = cam.right().as_vec3();
            let up = cam.up().as_vec3();
            for _ in 0..3 {
                let sx = rng.gen_range(-0.05f32..0.05);
                let sy = rng.gen_range(-0.05f32..0.05);
                let dir = (fwd + right * sx + up * sy).normalize();
                commands.spawn((
                    PbrBundle {
                        mesh: Mesh3d(proj_assets.sphere_sm.clone()),
                        material: MeshMaterial3d(proj_assets.mat_energy.clone()),
                        transform: Transform::from_translation(pos + fwd * 0.5),
                        ..default()
                    },
                    Projectile {
                        damage: dmg / 3.0, speed: 60.0, direction: dir,
                        lifetime: 2.0, is_explosive: false, explosion_radius: 0.0,
                        weapon_type: ProjectileOwner::EnergyBurst,
                        owner: None, piercing: true, gravity_affected: false, vertical_velocity: 0.0,
                    },
                ));
            }
            fired_ev.send(WeaponFiredEvent);
            msg_ev.send(UiMessageEvent {
                text: format!("Energy Burst! [{} ammo]", inv.slot8.ammo), duration: 1.5,
            });
        } else {
            msg_ev.send(UiMessageEvent { text: "Energy Burst cooling down!".to_string(), duration: 1.0 });
        }
    }

    // Slot 9 – Bomb
    if keyboard.just_pressed(KeyCode::Digit9) {
        if inv.slot9.can_fire() {
            let dmg = inv.slot9.effective_damage();
            inv.slot9.cooldown_timer = inv.slot9.cooldown;
            inv.slot9.ammo = inv.slot9.ammo.saturating_sub(1);
            commands.spawn((
                PbrBundle {
                    mesh: Mesh3d(proj_assets.sphere_lg.clone()),
                    material: MeshMaterial3d(proj_assets.mat_bomb.clone()),
                    transform: Transform::from_translation(pos + fwd * 0.5),
                    ..default()
                },
                Projectile {
                    damage: dmg, speed: 12.0, direction: fwd,
                    lifetime: 3.5, is_explosive: true, explosion_radius: 12.0,
                    weapon_type: ProjectileOwner::Bomb,
                    owner: None, piercing: false, gravity_affected: true, vertical_velocity: 0.1,
                },
            ));
            fired_ev.send(WeaponFiredEvent);
            msg_ev.send(UiMessageEvent {
                text: format!("BOMB DEPLOYED! [{} ammo]", inv.slot9.ammo), duration: 1.5,
            });
        } else {
            msg_ev.send(UiMessageEvent { text: "Bomb cooling down!".to_string(), duration: 1.0 });
        }
    }

    // Slot 0 – Combat Drone shot
    if keyboard.just_pressed(KeyCode::Digit0) {
        if inv.slot0.can_fire() {
            let dmg = inv.slot0.effective_damage();
            inv.slot0.cooldown_timer = inv.slot0.cooldown;
            inv.slot0.ammo = inv.slot0.ammo.saturating_sub(1);
            // Fire a burst of 5 drone shots slightly offset
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let right = cam.right().as_vec3();
            let up = cam.up().as_vec3();
            for _ in 0..5 {
                let sx = rng.gen_range(-0.08f32..0.08);
                let sy = rng.gen_range(-0.08f32..0.08);
                let dir = (fwd + right * sx + up * sy).normalize();
                commands.spawn((
                    PbrBundle {
                        mesh: Mesh3d(proj_assets.sphere_sm.clone()),
                        material: MeshMaterial3d(proj_assets.mat_drone_shot.clone()),
                        transform: Transform::from_translation(pos + fwd * 0.5),
                        ..default()
                    },
                    Projectile {
                        damage: dmg / 5.0, speed: 45.0, direction: dir,
                        lifetime: 3.0, is_explosive: false, explosion_radius: 0.0,
                        weapon_type: ProjectileOwner::Drone,
                        owner: None, piercing: false, gravity_affected: false, vertical_velocity: 0.0,
                    },
                ));
            }
            fired_ev.send(WeaponFiredEvent);
            msg_ev.send(UiMessageEvent {
                text: format!("Combat Drone! [{} charges]", inv.slot0.ammo), duration: 1.5,
            });
        } else {
            msg_ev.send(UiMessageEvent { text: "Drone recharging!".to_string(), duration: 1.0 });
        }
    }
}

// ── Projectile Update ─────────────────────────────────────────────────────────
fn projectile_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut proj_q: Query<(Entity, &mut Transform, &mut Projectile)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Health, &mut Damageable, &Enemy), Without<Projectile>>,
    mut enemy_damaged_ev: EventWriter<EnemyDamagedEvent>,
    mut enemy_killed_ev: EventWriter<EnemyKilledEvent>,
) {
    let dt = time.delta_secs();

    for (proj_entity, mut proj_transform, mut proj) in proj_q.iter_mut() {
        proj_transform.translation += proj.direction * proj.speed * dt;

        if proj.gravity_affected {
            proj.vertical_velocity -= 9.8 * dt;
            proj_transform.translation.y += proj.vertical_velocity * dt;
        }

        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            if proj.is_explosive {
                explode(&proj_transform.translation, proj.explosion_radius, proj.damage,
                    &mut enemy_q, &mut enemy_damaged_ev, &mut enemy_killed_ev);
            }
            commands.entity(proj_entity).despawn_recursive();
            continue;
        }

        if proj_transform.translation.y < 0.0 {
            if proj.is_explosive {
                explode(&proj_transform.translation, proj.explosion_radius, proj.damage,
                    &mut enemy_q, &mut enemy_damaged_ev, &mut enemy_killed_ev);
            }
            commands.entity(proj_entity).despawn_recursive();
            continue;
        }

        let mut hit = false;
        let mut explosion: Option<(Vec3, f32, f32)> = None;

        for (e_entity, e_transform, mut e_health, mut e_damageable, enemy) in enemy_q.iter_mut() {
            if !e_health.is_alive() { continue; }
            let dist = proj_transform.translation.distance(e_transform.translation);
            if dist < 1.5 {
                if proj.is_explosive {
                    explosion = Some((proj_transform.translation, proj.explosion_radius, proj.damage));
                    hit = true;
                    break;
                } else {
                    let info = DamageInfo::new(proj.damage, DamageType::Plasma);
                    let result = apply_damage(&mut e_health, &mut e_damageable, &info);
                    enemy_damaged_ev.send(EnemyDamagedEvent {
                        entity: e_entity,
                        damage: result.damage_amount,
                        position: e_transform.translation,
                    });
                    if result.was_killed {
                        enemy_killed_ev.send(EnemyKilledEvent {
                            enemy_type: enemy.enemy_type.as_str().to_string(),
                            credits: enemy.config.credits,
                            experience: enemy.config.experience_value,
                            position: e_transform.translation,
                        });
                    }
                }
                if !proj.piercing {
                    hit = true;
                    break;
                }
            }
        }

        if let Some((pos, radius, dmg)) = explosion {
            explode(&pos, radius, dmg, &mut enemy_q, &mut enemy_damaged_ev, &mut enemy_killed_ev);
        }
        if hit {
            commands.entity(proj_entity).despawn_recursive();
        }
    }
}

fn explode(
    center: &Vec3,
    radius: f32,
    base_damage: f32,
    enemy_q: &mut Query<(Entity, &Transform, &mut Health, &mut Damageable, &Enemy), Without<Projectile>>,
    damaged_ev: &mut EventWriter<EnemyDamagedEvent>,
    killed_ev: &mut EventWriter<EnemyKilledEvent>,
) {
    for (e_entity, e_transform, mut e_health, mut e_damageable, enemy) in enemy_q.iter_mut() {
        if !e_health.is_alive() { continue; }
        let dist = center.distance(e_transform.translation);
        if dist <= radius {
            let damage = area_damage_falloff(base_damage, dist, radius).max(1.0);
            let info = DamageInfo::new(damage, DamageType::Explosive);
            let result = apply_damage(&mut e_health, &mut e_damageable, &info);
            damaged_ev.send(EnemyDamagedEvent {
                entity: e_entity,
                damage: result.damage_amount,
                position: e_transform.translation,
            });
            if result.was_killed {
                killed_ev.send(EnemyKilledEvent {
                    enemy_type: enemy.enemy_type.as_str().to_string(),
                    credits: enemy.config.credits,
                    experience: enemy.config.experience_value,
                    position: e_transform.translation,
                });
            }
        }
    }
}

// ── Melee Combo ───────────────────────────────────────────────────────────────
const LIGHT_COMBO: &[(&str, f32, f32, f32)] = &[
    ("Jab",      15.0, 3.0, 0.4),
    ("Cross",    20.0, 4.0, 0.45),
    ("Uppercut", 30.0, 6.0, 0.6),
];

const HEAVY_COMBO: &[(&str, f32, f32, f32)] = &[
    ("Slam",  35.0, 8.0,  0.7),
    ("Sweep", 45.0, 10.0, 0.8),
];

fn melee_combo_system(
    gi:   Res<GameInput>,
    time: Res<Time>,
    mut commands: Commands,
    proj_assets: Res<ProjectileAssets>,
    mut player_q: Query<(&GlobalTransform, &mut MeleeCombo, &mut PlayerStateMachine), With<Player>>,
    cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut enemy_q: Query<(Entity, &Transform, &mut Health, &mut Damageable, &Enemy)>,
    mut combo_ev: EventWriter<ComboHitEvent>,
    mut finished_ev: EventWriter<ComboFinishedEvent>,
    mut damaged_ev: EventWriter<EnemyDamagedEvent>,
    mut killed_ev: EventWriter<EnemyKilledEvent>,
) {
    let dt = time.delta_secs();
    let Ok((_, mut combo, mut sm)) = player_q.get_single_mut() else { return };
    let Ok(cam) = cam_q.get_single() else { return };

    combo.light_timer = (combo.light_timer - dt).max(0.0);
    combo.heavy_timer = (combo.heavy_timer - dt).max(0.0);
    combo.active_timer = (combo.active_timer - dt).max(0.0);

    if gi.melee_light { combo.buffered_light = true; }
    if gi.melee_heavy { combo.buffered_heavy = true; }

    if combo.light_timer <= 0.0 { combo.light_index = 0; }
    if combo.heavy_timer <= 0.0 { combo.heavy_index = 0; }

    if combo.active_timer <= 0.0 && combo.is_attacking {
        combo.is_attacking = false;
        sm.transition(PlayerState::Idle);
    }
    if combo.is_attacking { return; }

    let do_light = combo.buffered_light;
    let do_heavy = combo.buffered_heavy;
    combo.buffered_light = false;
    combo.buffered_heavy = false;

    let cam_pos = cam.translation();
    let cam_fwd = cam.forward().as_vec3();

    if do_light && combo.light_index < LIGHT_COMBO.len() {
        let (name, base_damage, _knockback, duration) = LIGHT_COMBO[combo.light_index];
        let damage = base_damage * combo.damage_multiplier;

        execute_melee_hit(cam_pos, cam_fwd, 3.0, 2.5, damage, DamageType::Melee,
            &mut enemy_q, &mut damaged_ev, &mut killed_ev);

        // Melee flash VFX
        spawn_melee_flash(&mut commands, &proj_assets, cam_pos + cam_fwd * 2.5);

        combo_ev.send(ComboHitEvent {
            combo_name: "Light".to_string(),
            attack_name: name.to_string(),
            combo_index: combo.light_index,
        });
        combo.light_index = (combo.light_index + 1) % LIGHT_COMBO.len();
        combo.light_timer = 1.5;
        combo.active_timer = duration;
        combo.is_attacking = true;
        sm.force(PlayerState::Attacking);

        if combo.light_index == 0 {
            finished_ev.send(ComboFinishedEvent { combo_name: "Light".to_string() });
        }
    } else if do_heavy && combo.heavy_index < HEAVY_COMBO.len() {
        let (name, base_damage, _knockback, duration) = HEAVY_COMBO[combo.heavy_index];
        let damage = base_damage * combo.damage_multiplier;

        execute_melee_hit(cam_pos, cam_fwd, 4.5, 2.0, damage, DamageType::Melee,
            &mut enemy_q, &mut damaged_ev, &mut killed_ev);

        spawn_melee_flash(&mut commands, &proj_assets, cam_pos + cam_fwd * 2.0);

        combo_ev.send(ComboHitEvent {
            combo_name: "Heavy".to_string(),
            attack_name: name.to_string(),
            combo_index: combo.heavy_index,
        });
        combo.heavy_index = (combo.heavy_index + 1) % HEAVY_COMBO.len();
        combo.heavy_timer = 2.0;
        combo.active_timer = duration;
        combo.is_attacking = true;
        sm.force(PlayerState::Attacking);

        if combo.heavy_index == 0 {
            finished_ev.send(ComboFinishedEvent { combo_name: "Heavy".to_string() });
        }
    }
}

fn spawn_melee_flash(commands: &mut Commands, assets: &ProjectileAssets, position: Vec3) {
    commands.spawn((
        PbrBundle {
            mesh: Mesh3d(assets.flash_sphere.clone()),
            material: MeshMaterial3d(assets.mat_melee_flash.clone()),
            transform: Transform::from_translation(position),
            ..default()
        },
        HitParticle { lifetime: 0.12, max_lifetime: 0.12, velocity: Vec3::ZERO },
    ));
}

fn execute_melee_hit(
    origin: Vec3,
    forward: Vec3,
    radius: f32,
    offset: f32,
    damage: f32,
    damage_type: DamageType,
    enemy_q: &mut Query<(Entity, &Transform, &mut Health, &mut Damageable, &Enemy)>,
    damaged_ev: &mut EventWriter<EnemyDamagedEvent>,
    killed_ev: &mut EventWriter<EnemyKilledEvent>,
) {
    let hit_center = origin + forward * offset;
    for (e_entity, e_transform, mut health, mut damageable, enemy) in enemy_q.iter_mut() {
        if !health.is_alive() { continue; }
        if hit_center.distance(e_transform.translation) <= radius {
            let info = DamageInfo::new(damage, damage_type);
            let result = apply_damage(&mut health, &mut damageable, &info);
            damaged_ev.send(EnemyDamagedEvent {
                entity: e_entity,
                damage: result.damage_amount,
                position: e_transform.translation,
            });
            if result.was_killed {
                killed_ev.send(EnemyKilledEvent {
                    enemy_type: enemy.enemy_type.as_str().to_string(),
                    credits: enemy.config.credits,
                    experience: enemy.config.experience_value,
                    position: e_transform.translation,
                });
            }
        }
    }
}

// ── Beam Sabre ────────────────────────────────────────────────────────────────
fn beam_sabre_update_system(
    gi:   Res<GameInput>,
    time: Res<Time>,
    mut commands: Commands,
    proj_assets: Res<ProjectileAssets>,
    mut player_q: Query<(&mut BeamSabre, &mut PlayerStateMachine), With<Player>>,
    cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut enemy_q: Query<(Entity, &Transform, &mut Health, &mut Damageable, &Enemy)>,
    mut damaged_ev: EventWriter<EnemyDamagedEvent>,
    mut killed_ev: EventWriter<EnemyKilledEvent>,
) {
    let dt = time.delta_secs();
    let Ok((mut sabre, mut sm)) = player_q.get_single_mut() else { return };
    let Ok(cam) = cam_q.get_single() else { return };

    // Toggle sabre — T key or Mode button (Xbox Guide / PS Home)
    if gi.sabre_toggle {
        if !sabre.unlocked { return; }
        sabre.active = !sabre.active;
        return;
    }

    if !sabre.unlocked { return; }
    if !sabre.active { return; }

    sabre.cooldown_timer = (sabre.cooldown_timer - dt).max(0.0);

    // Continue active slash sequence
    if sabre.is_slashing {
        sabre.slash_timer -= dt;
        if sabre.slash_timer <= 0.0 {
            sabre.slash_index += 1;
            if sabre.slash_index < sabre.slash_count {
                execute_melee_hit(
                    cam.translation(), cam.forward().as_vec3(),
                    3.5, 2.5, sabre.slash_damage, DamageType::Melee,
                    &mut enemy_q, &mut damaged_ev, &mut killed_ev,
                );
                spawn_melee_flash(&mut commands, &proj_assets, cam.translation() + cam.forward().as_vec3() * 2.5);
                sabre.slash_timer = 0.25;
            } else {
                sabre.is_slashing = false;
                sabre.slash_index = 0;
                sm.transition(PlayerState::Idle);
            }
        }
        return;
    }

    // Begin new slash sequence on fire input
    if gi.fire_just && sabre.cooldown_timer <= 0.0 {
        sabre.is_slashing = true;
        sabre.slash_index = 0;
        sabre.cooldown_timer = sabre.cooldown;
        sabre.slash_timer = 0.25;
        sm.force(PlayerState::Attacking);

        // First slash hit immediately
        execute_melee_hit(
            cam.translation(), cam.forward().as_vec3(),
            3.5, 2.5, sabre.slash_damage, DamageType::Melee,
            &mut enemy_q, &mut damaged_ev, &mut killed_ev,
        );
        spawn_melee_flash(&mut commands, &proj_assets, cam.translation() + cam.forward().as_vec3() * 2.5);

        // Level 4+: fire dual wave projectiles
        if sabre.fires_dual_wave() {
            let fwd = cam.forward().as_vec3();
            let right = cam.right().as_vec3();
            for offset in [-0.4f32, 0.4] {
                let dir = (fwd + right * offset).normalize();
                commands.spawn((
                    PbrBundle {
                        mesh: Mesh3d(proj_assets.sphere_sm.clone()),
                        material: MeshMaterial3d(proj_assets.mat_melee_flash.clone()),
                        transform: Transform::from_translation(cam.translation() + fwd),
                        ..default()
                    },
                    Projectile {
                        damage: sabre.wave_damage, speed: 20.0, direction: dir,
                        lifetime: 1.5, is_explosive: sabre.has_aoe_splash(),
                        explosion_radius: if sabre.has_aoe_splash() { 4.0 } else { 0.0 },
                        weapon_type: ProjectileOwner::Player,
                        owner: None, piercing: sabre.is_piercing(),
                        gravity_affected: false, vertical_velocity: 0.0,
                    },
                ));
            }
        }
    }
}

// ── Hit Particles ─────────────────────────────────────────────────────────────
fn hit_particle_spawn_system(
    mut commands: Commands,
    proj_assets: Res<ProjectileAssets>,
    mut damaged_ev: EventReader<EnemyDamagedEvent>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for ev in damaged_ev.read() {
        let count = rng.gen_range(4usize..8);
        for _ in 0..count {
            let vel = Vec3::new(
                rng.gen_range(-5.0f32..5.0),
                rng.gen_range(3.0f32..9.0),
                rng.gen_range(-5.0f32..5.0),
            );
            commands.spawn((
                PbrBundle {
                    mesh: Mesh3d(proj_assets.sphere_sm.clone()),
                    material: MeshMaterial3d(proj_assets.mat_hit_particle.clone()),
                    transform: Transform::from_translation(ev.position + Vec3::Y * 0.5),
                    ..default()
                },
                HitParticle { lifetime: 0.45, max_lifetime: 0.45, velocity: vel },
            ));
        }
    }
}

fn particle_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Transform, &mut HitParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle) in q.iter_mut() {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        transform.translation += particle.velocity * dt;
        particle.velocity.y -= 18.0 * dt;
        // Shrink as lifetime drops
        let t = (particle.lifetime / particle.max_lifetime).max(0.05);
        transform.scale = Vec3::splat(t);
    }
}
