/// Unified input abstraction — keyboard/mouse + wired/wireless gamepad.
///
/// All game systems read `Res<GameInput>` instead of raw device resources.
/// Adding remapping, dead-zone tuning, or new devices is a one-file change.
///
/// Bevy 0.15 gamepad notes:
///   - `Gamepad` is a **component** on entities; no `Gamepads` resource.
///   - `gamepad.pressed(GamepadButton::South)` / `.just_pressed(…)` / `.get(GamepadAxis::…)`
///   - Button naming: LeftTrigger = LB/L1 (bumper), LeftTrigger2 = LT/L2 (analog trigger)
///
/// Controller layout (Xbox / generic HID — PlayStation labels in comments):
///
///   Left  stick      → move
///   Right stick      → look
///   South  (A / ✕)  → jump / jetpack (hold)
///   East   (B / ○)  → dodge
///   West   (X / □)  → reload
///   North  (Y / △)  → parry
///   LT  (L2)         → aim-down-sights
///   RT  (R2)         → fire
///   LB  (L1)         → sprint
///   RB  (R1)         → next weapon
///   L3               → melee heavy
///   R3               → melee light
///   D-Pad Left       → previous weapon
///   D-Pad Right      → next weapon
///   D-Pad Up         → enter vehicle / jet
///   D-Pad Down       → interact
///   Select  (Share)  → crafting menu
///   Start   (Options)→ pause / menu
///   Mode    (Guide)  → beam sabre toggle

use bevy::prelude::*;
use bevy::input::gamepad::{Gamepad, GamepadButton, GamepadAxis};
use bevy::input::mouse::MouseMotion;

use crate::resources::GameSettings;

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameInput>()
            .add_systems(PreUpdate, update_game_input);
    }
}

// ── Tuning ────────────────────────────────────────────────────────────────────
const DEADZONE: f32        = 0.18;
const STICK_LOOK_RATE: f32 = std::f32::consts::PI * 1.5; // 270°/s at full deflection

// ── GameInput resource ────────────────────────────────────────────────────────
#[derive(Resource, Default)]
pub struct GameInput {
    // ── Analogue ─────────────────────────────────────────────────────────────
    /// Normalised movement intent (player-local XZ), range −1..1 per axis.
    pub move_axis:  Vec2,
    /// Look rotation delta for this frame in radians (already scaled).
    pub look_delta: Vec2,

    // ── Held ─────────────────────────────────────────────────────────────────
    pub fire:    bool,
    pub aim:     bool,
    pub sprint:  bool,
    pub jetpack: bool,

    // ── Just-pressed (cleared each frame) ────────────────────────────────────
    pub jump:          bool,
    pub fire_just:     bool,
    pub dodge:         bool,
    pub reload:        bool,
    pub parry:         bool,
    pub interact:      bool,
    pub melee_light:   bool,
    pub melee_heavy:   bool,
    pub crafting:      bool,
    pub pause:         bool,
    pub weapon_next:   bool,
    pub weapon_prev:   bool,
    pub enter_vehicle: bool,
    pub open_map:      bool,
    pub sabre_toggle:  bool,
    /// Direct slot key (0-based) if pressed; `None` otherwise.
    pub weapon_slot:   Option<usize>,

    // ── Metadata ─────────────────────────────────────────────────────────────
    pub gamepad_active: bool,
}

// ── Update system ─────────────────────────────────────────────────────────────
fn update_game_input(
    keyboard:   Res<ButtonInput<KeyCode>>,
    mouse_btn:  Res<ButtonInput<MouseButton>>,
    mut mouse_ev: EventReader<MouseMotion>,
    gamepads:   Query<&Gamepad>,
    time:       Res<Time>,
    settings:   Res<GameSettings>,
    mut gi:     ResMut<GameInput>,
) {
    let dt   = time.delta_secs();
    let sens = settings.mouse_sensitivity;

    // Clear all flags so "just-pressed" semantics work correctly.
    *gi = GameInput::default();

    // ── First connected gamepad (works for USB and Bluetooth) ─────────────────
    let gp: Option<&Gamepad> = gamepads.iter().next();
    gi.gamepad_active = gp.is_some();

    // ── Helpers ───────────────────────────────────────────────────────────────
    let btn_held = |b: GamepadButton| gp.map(|g| g.pressed(b)).unwrap_or(false);
    let btn_just = |b: GamepadButton| gp.map(|g| g.just_pressed(b)).unwrap_or(false);
    let axis_val = |a: GamepadAxis| -> f32 {
        gp.and_then(|g| g.get(a)).unwrap_or(0.0)
    };
    let apply_deadzone = |v: Vec2| if v.length() < DEADZONE { Vec2::ZERO } else { v };

    // ── Movement (left stick + WASD) ──────────────────────────────────────────
    let kb_move = Vec2::new(
        (keyboard.pressed(KeyCode::KeyD) as i32 - keyboard.pressed(KeyCode::KeyA) as i32) as f32,
        (keyboard.pressed(KeyCode::KeyW) as i32 - keyboard.pressed(KeyCode::KeyS) as i32) as f32,
    ).normalize_or_zero();

    let gp_move = apply_deadzone(Vec2::new(
        axis_val(GamepadAxis::LeftStickX),
        axis_val(GamepadAxis::LeftStickY),
    ));

    gi.move_axis = if gp_move.length_squared() > 0.001 { gp_move } else { kb_move };

    // ── Look delta (right stick + mouse) ─────────────────────────────────────
    let mut raw_mouse = Vec2::ZERO;
    for ev in mouse_ev.read() {
        raw_mouse += ev.delta;
    }
    let mouse_look = raw_mouse * sens;

    let raw_stick = apply_deadzone(Vec2::new(
        axis_val(GamepadAxis::RightStickX),
        -axis_val(GamepadAxis::RightStickY),
    ));
    // Quadratic curve: small nudges = precision, full deflection = fast sweep.
    let curved_stick = raw_stick * raw_stick.length();
    let gp_look = curved_stick * STICK_LOOK_RATE * dt;

    gi.look_delta = mouse_look + gp_look;

    // ── Fire — RT (R2) / LMB ─────────────────────────────────────────────────
    gi.fire      = mouse_btn.pressed(MouseButton::Left)      || btn_held(GamepadButton::RightTrigger2);
    gi.fire_just = mouse_btn.just_pressed(MouseButton::Left) || btn_just(GamepadButton::RightTrigger2);

    // ── Aim — LT (L2) / RMB ──────────────────────────────────────────────────
    gi.aim = mouse_btn.pressed(MouseButton::Right) || btn_held(GamepadButton::LeftTrigger2);

    // ── Sprint — LB (L1) / Shift ─────────────────────────────────────────────
    gi.sprint = keyboard.pressed(KeyCode::ShiftLeft)
             || keyboard.pressed(KeyCode::ShiftRight)
             || btn_held(GamepadButton::LeftTrigger);

    // ── Jump / Jetpack — South (A/✕) / Space ─────────────────────────────────
    gi.jump    = keyboard.just_pressed(KeyCode::Space) || btn_just(GamepadButton::South);
    gi.jetpack = keyboard.pressed(KeyCode::Space)      || btn_held(GamepadButton::South);

    // ── Dodge — East (B/○) / Q ───────────────────────────────────────────────
    gi.dodge = keyboard.just_pressed(KeyCode::KeyQ) || btn_just(GamepadButton::East);

    // ── Reload — West (X/□) / R ──────────────────────────────────────────────
    gi.reload = keyboard.just_pressed(KeyCode::KeyR) || btn_just(GamepadButton::West);

    // ── Parry — North (Y/△) / F ──────────────────────────────────────────────
    gi.parry = keyboard.just_pressed(KeyCode::KeyF) || btn_just(GamepadButton::North);

    // ── Interact — D-Down / E ────────────────────────────────────────────────
    gi.interact = keyboard.just_pressed(KeyCode::KeyE) || btn_just(GamepadButton::DPadDown);

    // ── Melee light — R3 / V ─────────────────────────────────────────────────
    gi.melee_light = keyboard.just_pressed(KeyCode::KeyV) || btn_just(GamepadButton::RightThumb);

    // ── Melee heavy — L3 / B ─────────────────────────────────────────────────
    gi.melee_heavy = keyboard.just_pressed(KeyCode::KeyB) || btn_just(GamepadButton::LeftThumb);

    // ── Crafting — Select / C ────────────────────────────────────────────────
    gi.crafting = keyboard.just_pressed(KeyCode::KeyC) || btn_just(GamepadButton::Select);

    // ── Pause — Start / Escape ────────────────────────────────────────────────
    gi.pause = keyboard.just_pressed(KeyCode::Escape) || btn_just(GamepadButton::Start);

    // ── Weapon cycle — RB (R1) / D-Right / ] and D-Left / [ ─────────────────
    gi.weapon_next = keyboard.just_pressed(KeyCode::BracketRight)
                   || btn_just(GamepadButton::RightTrigger)
                   || btn_just(GamepadButton::DPadRight);
    gi.weapon_prev = keyboard.just_pressed(KeyCode::BracketLeft)
                   || btn_just(GamepadButton::DPadLeft);

    // ── Direct weapon slots — Digit 1-6 ──────────────────────────────────────
    gi.weapon_slot = if keyboard.just_pressed(KeyCode::Digit1) { Some(0) }
        else if keyboard.just_pressed(KeyCode::Digit2) { Some(1) }
        else if keyboard.just_pressed(KeyCode::Digit3) { Some(2) }
        else if keyboard.just_pressed(KeyCode::Digit4) { Some(3) }
        else if keyboard.just_pressed(KeyCode::Digit5) { Some(4) }
        else if keyboard.just_pressed(KeyCode::Digit6) { Some(5) }
        else { None };

    // ── Vehicle — D-Up / J ────────────────────────────────────────────────────
    gi.enter_vehicle = keyboard.just_pressed(KeyCode::KeyJ) || btn_just(GamepadButton::DPadUp);

    // ── Map / Motorcycle — M ─────────────────────────────────────────────────
    gi.open_map = keyboard.just_pressed(KeyCode::KeyM);

    // ── Beam sabre toggle — Mode (Guide) / T ─────────────────────────────────
    gi.sabre_toggle = keyboard.just_pressed(KeyCode::KeyT) || btn_just(GamepadButton::Mode);
}
