use bevy::prelude::*;

use crate::state::AppState;
use crate::events::*;
use crate::components::player::*;
use crate::components::weapon::*;
use crate::components::enemy::Enemy;
use crate::damage::{Health, Damageable, DamageInfo, DamageType, apply_damage, area_damage_falloff};

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                weapon_select_system,
                weapon_fire_system,
                weapon_reload_system,
                projectile_update_system,
                melee_combo_system,
                beam_sabre_update_system,
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// ── Weapon Select ─────────────────────────────────────────────────────────────
fn weapon_select_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut WeaponInventory, With<Player>>,
    mut switched_ev: EventWriter<WeaponSwitchedEvent>,
) {
    let Ok(mut inv) = player_q.get_single_mut() else { return };

    let prev = inv.active_slot;
    let slot = if keyboard.just_pressed(KeyCode::Digit1) { Some(0) }
        else if keyboard.just_pressed(KeyCode::Digit2) { Some(1) }
        else if keyboard.just_pressed(KeyCode::Digit3) { Some(2) }
        else if keyboard.just_pressed(KeyCode::Digit4) { Some(3) }
        else if keyboard.just_pressed(KeyCode::Digit5) { Some(4) }
        else if keyboard.just_pressed(KeyCode::Digit6) { Some(5) }
        else { None };

    if let Some(s) = slot {
        inv.active_slot = s;
        if s != prev {
            switched_ev.send(WeaponSwitchedEvent {
                weapon_name: inv.active().weapon_type.display_name().to_string(),
            });
        }
    }
}

// ── Fire ─────────────────────────────────────────────────────────────────────
fn weapon_fire_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut commands: Commands,
    mut player_q: Query<(&mut WeaponInventory, &Transform, &mut PlayerStateMachine, &PlayerStats), With<Player>>,
    mut cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut fired_ev: EventWriter<WeaponFiredEvent>,
) {
    let dt = time.delta_secs();
    let Ok((mut inv, _, mut sm, _)) = player_q.get_single_mut() else { return };
    let Ok(cam_transform) = cam_q.get_single_mut() else { return };

    let firing = mouse.pressed(MouseButton::Left);
    let just_fired = mouse.just_pressed(MouseButton::Left);
    let weapon = inv.active_mut();

    // Tick fire timer
    weapon.fire_timer = (weapon.fire_timer - dt).max(0.0);

    let should_fire = if weapon.automatic { firing } else { just_fired };
    if !should_fire || !weapon.can_fire() { return; }

    let pos = cam_transform.translation();
    let forward = cam_transform.forward().as_vec3();
    let right = cam_transform.right().as_vec3();
    let up = cam_transform.up().as_vec3();

    let damage = weapon.damage;
    let speed = weapon.speed;
    let spread = weapon.spread;
    let pellets = weapon.pellets;
    let is_explosive = weapon.is_explosive;
    let explosion_radius = weapon.explosion_radius;
    let w_type = match weapon.weapon_type {
        WeaponType::Grenade => ProjectileOwner::Player,
        _ => ProjectileOwner::Player,
    };
    let gravity_affected = weapon.weapon_type == WeaponType::Grenade;

    weapon.ammo = weapon.ammo.saturating_sub(1);
    weapon.fire_timer = weapon.fire_rate;

    for i in 0..pellets {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let sx = rng.gen_range(-spread..spread);
        let sy = rng.gen_range(-spread..spread);
        let dir = (forward + right * sx + up * sy).normalize();

        commands.spawn((
            Projectile {
                damage,
                speed,
                direction: dir,
                lifetime: 3.0,
                is_explosive,
                explosion_radius,
                weapon_type: w_type,
                owner: None,
                piercing: false,
                gravity_affected,
                vertical_velocity: if gravity_affected { 0.2 } else { 0.0 },
            },
            Transform::from_translation(pos + forward * 0.5),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
    }

    sm.transition(PlayerState::Attacking);
    fired_ev.send(WeaponFiredEvent);
}

// ── Reload ────────────────────────────────────────────────────────────────────
fn weapon_reload_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut WeaponInventory, With<Player>>,
    mut reload_ev: EventWriter<WeaponReloadedEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok(mut inv) = player_q.get_single_mut() {
            inv.active_mut().reload();
            reload_ev.send(WeaponReloadedEvent);
        }
    }
}

// ── Special Weapon Fire ───────────────────────────────────────────────────────
pub fn fire_special_weapon_slot7(inv: &mut SpecialWeaponInventory, pos: Vec3, dir: Vec3) -> Option<Projectile> {
    let sw = &mut inv.slot7;
    if !sw.can_fire() { return None; }
    sw.cooldown_timer = sw.cooldown;
    sw.ammo = sw.ammo.saturating_sub(1);
    Some(Projectile {
        damage: sw.effective_damage(),
        speed: 25.0,
        direction: dir,
        lifetime: 5.0,
        is_explosive: true,
        explosion_radius: 5.0,
        weapon_type: ProjectileOwner::Missile,
        owner: None,
        piercing: false,
        gravity_affected: false,
        vertical_velocity: 0.0,
    })
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
        // Move
        proj_transform.translation += proj.direction * proj.speed * dt;

        // Grenade arc
        if proj.gravity_affected {
            proj.vertical_velocity -= 9.8 * dt;
            proj_transform.translation.y += proj.vertical_velocity * dt;
        }

        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(proj_entity).despawn_recursive();
            continue;
        }

        // Ground clip
        if proj_transform.translation.y < 0.0 {
            if proj.is_explosive {
                explode(&proj_transform.translation, proj.explosion_radius, proj.damage,
                    &mut enemy_q, &mut enemy_damaged_ev, &mut enemy_killed_ev);
            }
            commands.entity(proj_entity).despawn_recursive();
            continue;
        }

        // Enemy hit detection
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
    // (name, damage, knockback, duration)
    ("Jab",     15.0, 3.0, 0.4),
    ("Cross",   20.0, 4.0, 0.45),
    ("Uppercut",30.0, 6.0, 0.6),
];

const HEAVY_COMBO: &[(&str, f32, f32, f32)] = &[
    ("Slam",  35.0, 8.0,  0.7),
    ("Sweep", 45.0, 10.0, 0.8),
];

fn melee_combo_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
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

    // Buffer inputs
    if keyboard.just_pressed(KeyCode::KeyV) {
        if combo.is_attacking { combo.buffered_light = true; } else { combo.buffered_light = true; }
    }
    if keyboard.just_pressed(KeyCode::KeyB) {
        if combo.is_attacking { combo.buffered_heavy = true; } else { combo.buffered_heavy = true; }
    }

    // Reset combos on inactivity
    if combo.light_timer <= 0.0 { combo.light_index = 0; }
    if combo.heavy_timer <= 0.0 { combo.heavy_index = 0; }

    if combo.active_timer <= 0.0 && combo.is_attacking {
        combo.is_attacking = false;
        sm.transition(PlayerState::Idle);
    }

    if combo.is_attacking { return; }

    // Execute buffered or new attack
    let do_light = combo.buffered_light;
    let do_heavy = combo.buffered_heavy;
    combo.buffered_light = false;
    combo.buffered_heavy = false;

    let cam_pos = cam.translation();
    let cam_fwd = cam.forward().as_vec3();

    if do_light && combo.light_index < LIGHT_COMBO.len() {
        let (name, base_damage, knockback, duration) = LIGHT_COMBO[combo.light_index];
        let damage = base_damage * combo.damage_multiplier;

        execute_melee_hit(cam_pos, cam_fwd, 3.0, 2.5, damage, DamageType::Melee,
            knockback, &mut enemy_q, &mut damaged_ev, &mut killed_ev);

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
        let (name, base_damage, knockback, duration) = HEAVY_COMBO[combo.heavy_index];
        let damage = base_damage * combo.damage_multiplier;

        execute_melee_hit(cam_pos, cam_fwd, 4.5, 2.0, damage, DamageType::Melee,
            knockback, &mut enemy_q, &mut damaged_ev, &mut killed_ev);

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

fn execute_melee_hit(
    origin: Vec3,
    forward: Vec3,
    radius: f32,
    offset: f32,
    damage: f32,
    damage_type: DamageType,
    _knockback: f32,
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
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<(&mut BeamSabre, &mut PlayerStateMachine), With<Player>>,
    cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut enemy_q: Query<(Entity, &Transform, &mut Health, &mut Damageable, &Enemy)>,
    mut damaged_ev: EventWriter<EnemyDamagedEvent>,
    mut killed_ev: EventWriter<EnemyKilledEvent>,
) {
    let dt = time.delta_secs();
    let Ok((mut sabre, mut sm)) = player_q.get_single_mut() else { return };
    let Ok(cam) = cam_q.get_single() else { return };

    // Toggle
    if keyboard.just_pressed(KeyCode::KeyT) {
        sabre.active = !sabre.active;
    }

    if !sabre.active { return; }

    sabre.cooldown_timer = (sabre.cooldown_timer - dt).max(0.0);

    // Active attack (left click while sabre active)
    if keyboard.just_pressed(KeyCode::KeyT) { return; } // already handled toggle

    if sabre.is_slashing {
        sabre.slash_timer -= dt;
        if sabre.slash_timer <= 0.0 {
            sabre.slash_index += 1;
            if sabre.slash_index < sabre.slash_count {
                // Next slash
                execute_melee_hit(
                    cam.translation(), cam.forward().as_vec3(),
                    3.5, 2.5, sabre.slash_damage, DamageType::Melee, 0.0,
                    &mut enemy_q, &mut damaged_ev, &mut killed_ev,
                );
                sabre.slash_timer = 0.25;
            } else {
                // Combo finished
                sabre.is_slashing = false;
                sabre.slash_index = 0;
                sm.transition(PlayerState::Idle);
            }
        }
    }
}
