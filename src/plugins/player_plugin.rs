use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

use crate::state::AppState;
use crate::events::*;
use crate::damage::{Health, Damageable, DamageInfo, DamageType, apply_damage};
use crate::components::player::*;
use crate::components::weapon::*;
use crate::components::armor::ArmorSet;
use crate::components::inventory::Inventory;
use crate::resources::{GameSettings, CameraShake};

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Playing), (spawn_player, grab_cursor))
            .add_systems(OnExit(AppState::Playing), release_cursor)
            .add_systems(
                Update,
                (
                    player_look,
                    camera_shake_system,
                    player_movement,
                    player_dodge_update,
                    player_parry_update,
                    player_state_update,
                    player_stamina_regen,
                    player_invulnerability_update,
                    player_level_up,
                    player_died_check,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

// ── Spawn ─────────────────────────────────────────────────────────────────────
fn spawn_player(mut commands: Commands) {
    // Physics body (capsule, matches original 1.4-unit-tall character)
    let player = commands
        .spawn((
            Player,
            Transform::from_xyz(350.0, 15.0, 150.0),
            GlobalTransform::default(),
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(0.6, 0.35),
            KinematicCharacterController {
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.02),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(0.5),
                    min_width: CharacterLength::Absolute(0.2),
                    include_dynamic_bodies: false,
                }),
                snap_to_ground: Some(CharacterLength::Absolute(0.2)),
                ..default()
            },
            KinematicCharacterControllerOutput::default(),
            PlayerStats::default(),
            PlayerMovement::default(),
        ))
        .insert((
            JetpackState::default(),
            DodgeState::new(),
            ParryState::new(),
            PlayerStateMachine::default(),
            Health::new(100.0),
            Damageable::default(),
        ))
        .insert((
            ArmorSet::default(),
            Inventory::default(),
            WeaponInventory::default(),
            SpecialWeaponInventory::default(),
            BeamSabre::default(),
            MeleeCombo::new(),
        ))
        .id();

    // First-person camera (child of player body)
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.8, 0.0),
                camera: Camera { hdr: true, ..default() },
                ..default()
            },
            PlayerCamera,
            CameraPitch::default(),
            bevy::core_pipeline::bloom::Bloom {
                intensity: 0.25,
                ..default()
            },
            DistanceFog {
                color: Color::srgba(0.02, 0.02, 0.08, 1.0),
                falloff: FogFalloff::ExponentialSquared { density: 0.0015 },
                ..default()
            },
        ))
        .set_parent(player);
}

fn grab_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn release_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

// ── Mouse Look ────────────────────────────────────────────────────────────────
fn player_look(
    settings: Res<GameSettings>,
    mut mouse_events: EventReader<MouseMotion>,
    mut player_q: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut cam_q: Query<(&mut Transform, &mut CameraPitch), (With<PlayerCamera>, Without<Player>)>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse_events.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO { return; }

    let sens = settings.mouse_sensitivity;

    if let Ok(mut pt) = player_q.get_single_mut() {
        pt.rotate_y(-delta.x * sens);
    }

    if let Ok((mut ct, mut pitch)) = cam_q.get_single_mut() {
        pitch.0 = (pitch.0 - delta.y * sens)
            .clamp(-std::f32::consts::FRAC_PI_2 * 0.9, std::f32::consts::FRAC_PI_2 * 0.9);
        ct.rotation = Quat::from_rotation_x(pitch.0);
    }
}

// ── Camera Shake ──────────────────────────────────────────────────────────────
fn camera_shake_system(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut cam_q: Query<&mut Transform, With<PlayerCamera>>,
    mut damage_ev: EventReader<PlayerDamagedEvent>,
) {
    for ev in damage_ev.read() {
        let trauma = (ev.amount / 25.0).clamp(0.12, 0.65);
        shake.add_trauma(trauma);
    }

    shake.trauma = (shake.trauma - time.delta_secs() * 2.0).max(0.0);

    let Ok(mut cam) = cam_q.get_single_mut() else { return };

    if shake.trauma > 0.01 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mag = shake.trauma * shake.trauma * 0.18;
        cam.translation = Vec3::new(
            rng.gen_range(-1.0f32..1.0) * mag,
            0.8 + rng.gen_range(-0.5f32..0.5) * mag,
            rng.gen_range(-1.0f32..1.0) * mag,
        );
    } else {
        cam.translation = Vec3::new(0.0, 0.8, 0.0);
    }
}

// ── Movement & Physics ────────────────────────────────────────────────────────
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<
        (
            &mut KinematicCharacterController,
            &KinematicCharacterControllerOutput,
            &mut PlayerMovement,
            &mut PlayerStats,
            &mut JetpackState,
            &mut DodgeState,
            &Transform,
            &mut PlayerStateMachine,
        ),
        With<Player>,
    >,
) {
    let dt = time.delta_secs();
    let Ok((
        mut controller,
        output,
        mut movement,
        mut stats,
        mut jetpack,
        mut dodge,
        transform,
        mut state,
    )) = player_q.get_single_mut() else { return };

    movement.is_grounded = output.grounded;

    // Regenerate jetpack fuel when grounded
    if movement.is_grounded {
        jetpack.fuel = (jetpack.fuel + jetpack.regen_rate * dt).min(jetpack.max_fuel);
        movement.velocity.y = movement.velocity.y.max(0.0); // reset fall
    }

    // Horizontal input
    let fwd = transform.forward().as_vec3().with_y(0.0).normalize_or_zero();
    let right = transform.right().as_vec3().with_y(0.0).normalize_or_zero();
    let mut input = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { input += fwd; }
    if keyboard.pressed(KeyCode::KeyS) { input -= fwd; }
    if keyboard.pressed(KeyCode::KeyA) { input -= right; }
    if keyboard.pressed(KeyCode::KeyD) { input += right; }
    let input = input.normalize_or_zero();

    let sprinting = keyboard.pressed(KeyCode::ShiftLeft) && stats.stamina > 0.0 && input.length_squared() > 0.0;
    let speed = if sprinting { movement.sprint_speed } else { movement.walk_speed };

    // Drain stamina while sprinting
    if sprinting {
        stats.stamina = (stats.stamina - 15.0 * dt).max(0.0);
    }

    // Jump
    if keyboard.just_pressed(KeyCode::Space) && movement.is_grounded {
        movement.velocity.y = movement.jump_force;
        movement.is_grounded = false;
        state.transition(PlayerState::Jetpack);
    }

    // Jetpack (hold Space while airborne)
    if keyboard.pressed(KeyCode::Space) && !movement.is_grounded && jetpack.fuel > 0.0 {
        movement.velocity.y = (movement.velocity.y + jetpack.force).min(jetpack.max_vertical_vel);
        jetpack.fuel -= jetpack.fuel_cost_per_sec * dt;
        jetpack.fuel = jetpack.fuel.max(0.0);
        jetpack.is_active = true;
        state.transition(PlayerState::Jetpack);
    } else {
        jetpack.is_active = false;
    }

    // Gravity
    if !movement.is_grounded {
        movement.velocity.y -= movement.gravity;
        movement.velocity.y = movement.velocity.y.max(-2.0);
    }

    // Horizontal movement
    let mut h_vel = if movement.is_grounded {
        let v = input * speed;
        movement.ground_velocity = v;
        v
    } else {
        // Air momentum: blend input influence into carried ground velocity
        let target = input * speed;
        movement.ground_velocity = movement.ground_velocity.lerp(target, 0.15);
        movement.ground_velocity
    };

    // Dodge override
    if dodge.is_dodging {
        h_vel = dodge.dodge_direction * dodge.dodge_speed;
    }

    // Compose final translation (delta per frame)
    let translation = (h_vel + Vec3::new(0.0, movement.velocity.y, 0.0)) * dt * 60.0;
    controller.translation = Some(translation);

    // Update player state machine
    if movement.is_grounded && !dodge.is_dodging {
        if input.length_squared() > 0.01 {
            if sprinting { state.transition(PlayerState::Sprinting); }
            else { state.transition(PlayerState::Moving); }
        } else {
            state.transition(PlayerState::Idle);
        }
    }
}

// ── Dodge Update ──────────────────────────────────────────────────────────────
fn player_dodge_update(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<
        (&mut DodgeState, &mut PlayerStats, &mut Damageable, &Transform, &mut PlayerStateMachine),
        With<Player>,
    >,
    mut dodge_ev: EventWriter<PlayerDodgeEvent>,
) {
    let dt = time.delta_secs();
    let Ok((mut dodge, mut stats, mut damageable, transform, mut state)) = player_q.get_single_mut() else { return };

    dodge.cooldown_timer = (dodge.cooldown_timer - dt).max(0.0);

    if dodge.is_dodging {
        dodge.dodge_timer -= dt;
        damageable.is_invulnerable = true;
        if dodge.dodge_timer <= 0.0 {
            dodge.is_dodging = false;
            damageable.is_invulnerable = false;
            state.transition(PlayerState::Idle);
        }
    }

    // Trigger dodge
    if keyboard.just_pressed(KeyCode::KeyQ)
        && !dodge.is_dodging
        && dodge.cooldown_timer <= 0.0
        && stats.stamina >= dodge.dodge_cost
        && state.current != PlayerState::Dead
    {
        let fwd = transform.forward().as_vec3().with_y(0.0).normalize_or_zero();
        dodge.dodge_direction = if fwd.length_squared() > 0.0 { -fwd } else { -fwd };
        dodge.is_dodging = true;
        dodge.dodge_timer = dodge.dodge_duration;
        dodge.cooldown_timer = dodge.dodge_cooldown;
        stats.stamina -= dodge.dodge_cost;
        state.force(PlayerState::Dodging);
        dodge_ev.send(PlayerDodgeEvent);
    }
}

// ── Parry Update ──────────────────────────────────────────────────────────────
fn player_parry_update(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<(&mut ParryState, &mut PlayerStateMachine), With<Player>>,
) {
    let dt = time.delta_secs();
    let Ok((mut parry, mut state)) = player_q.get_single_mut() else { return };

    parry.cooldown_timer = (parry.cooldown_timer - dt).max(0.0);

    if parry.is_parrying {
        parry.parry_timer -= dt;
        if parry.parry_timer <= 0.0 {
            parry.is_parrying = false;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyF)
        && !parry.is_parrying
        && parry.cooldown_timer <= 0.0
        && state.current != PlayerState::Dead
    {
        parry.is_parrying = true;
        parry.parry_timer = parry.parry_window;
        parry.cooldown_timer = parry.parry_cooldown;
    }
}

// ── State Update ──────────────────────────────────────────────────────────────
fn player_state_update(time: Res<Time>, mut q: Query<&mut PlayerStateMachine, With<Player>>) {
    let dt = time.delta_secs();
    if let Ok(mut sm) = q.get_single_mut() {
        sm.timer += dt;
    }
}

// ── Stamina Regen ─────────────────────────────────────────────────────────────
fn player_stamina_regen(
    time: Res<Time>,
    mut q: Query<(&mut PlayerStats, &DodgeState), With<Player>>,
    mut ev: EventWriter<PlayerStaminaChangedEvent>,
) {
    let dt = time.delta_secs();
    if let Ok((mut stats, dodge)) = q.get_single_mut() {
        if !dodge.is_dodging && stats.stamina < stats.max_stamina {
            stats.stamina = (stats.stamina + 10.0 * dt).min(stats.max_stamina);
            ev.send(PlayerStaminaChangedEvent { stamina: stats.stamina });
        }
    }
}

// ── Invulnerability Update ────────────────────────────────────────────────────
fn player_invulnerability_update(
    time: Res<Time>,
    mut q: Query<&mut Damageable, With<Player>>,
) {
    let dt = time.delta_secs();
    if let Ok(mut dmg) = q.get_single_mut() {
        if dmg.invulnerability_timer > 0.0 {
            dmg.invulnerability_timer -= dt;
            if dmg.invulnerability_timer <= 0.0 {
                dmg.is_invulnerable = false;
                dmg.invulnerability_timer = 0.0;
            }
        }
    }
}

// ── Level Up ──────────────────────────────────────────────────────────────────
fn player_level_up(
    mut q: Query<(&mut PlayerStats, &mut Health), With<Player>>,
    mut level_ev: EventWriter<PlayerLevelUpEvent>,
) {
    let Ok((mut stats, mut health)) = q.get_single_mut() else { return };
    let xp_needed = stats.xp_for_next_level();
    if stats.experience >= xp_needed {
        stats.experience -= xp_needed;
        stats.level += 1;
        stats.max_health += 10.0;
        stats.max_stamina += 5.0;
        health.max = stats.max_health;
        health.current = health.max; // full heal on level-up
        stats.stamina = stats.max_stamina;
        level_ev.send(PlayerLevelUpEvent { level: stats.level });
    }
}

// ── Death Check ───────────────────────────────────────────────────────────────
fn player_died_check(
    mut q: Query<(&Health, &mut PlayerStateMachine), With<Player>>,
    mut died_ev: EventWriter<PlayerDiedEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok((health, mut sm)) = q.get_single_mut() else { return };
    if !health.is_alive() && sm.current != PlayerState::Dead {
        sm.force(PlayerState::Dead);
        died_ev.send(PlayerDiedEvent);
        next_state.set(AppState::GameOver);
    }
}

// ── Public helpers (called from other plugins) ────────────────────────────────

/// Apply damage to the player, respecting parry and armor.
pub fn damage_player(
    health: &mut Health,
    damageable: &mut Damageable,
    stats: &mut PlayerStats,
    parry: &mut ParryState,
    armor_set: &ArmorSet,
    info: &DamageInfo,
    damaged_ev: &mut EventWriter<PlayerDamagedEvent>,
    parry_ev: &mut EventWriter<PlayerParryEvent>,
) {
    if !health.is_alive() || damageable.is_invulnerable { return; }

    // Parry window
    if parry.is_parrying {
        parry.is_parrying = false;
        parry_ev.send(PlayerParryEvent { success: true });
        return;
    }

    // Armor reduces before health
    let armor_reduced = armor_set.calculate_damage_reduction(info.amount);

    // Armor absorbs 70 % of remaining damage
    let armor_absorb = armor_reduced * 0.7;
    let health_portion = armor_reduced * 0.3;

    stats.armor = (stats.armor - armor_absorb).max(0.0);

    let result = apply_damage(health, damageable, &DamageInfo {
        amount: health_portion,
        ..info.clone()
    });

    // Post-hit invulnerability
    damageable.is_invulnerable = true;
    damageable.invulnerability_timer = 0.2;

    damaged_ev.send(PlayerDamagedEvent {
        amount: result.damage_amount,
        remaining: health.current,
    });
}
