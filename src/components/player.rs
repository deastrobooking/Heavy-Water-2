use bevy::prelude::*;

// ── Marker ────────────────────────────────────────────────────────────────────
/// Marks the player entity (the physics body).
#[derive(Component, Default)]
pub struct Player;

/// Marks the first-person camera child entity.
#[derive(Component, Default)]
pub struct PlayerCamera;

// ── Stats ─────────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct PlayerStats {
    pub max_health: f32,
    pub armor: f32,
    pub max_armor: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub credits: u32,
    pub experience: u32,
    pub level: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            armor: 100.0,
            max_armor: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            credits: 0,
            experience: 0,
            level: 1,
        }
    }
}

impl PlayerStats {
    pub fn xp_for_next_level(&self) -> u32 {
        self.level * 100
    }
}

// ── Movement ──────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct PlayerMovement {
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    /// Last horizontal velocity while grounded (for air momentum)
    pub ground_velocity: Vec3,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            walk_speed: 0.3,
            sprint_speed: 0.55,
            jump_force: 0.5,
            gravity: 0.02,
            velocity: Vec3::ZERO,
            is_grounded: true,
            ground_velocity: Vec3::ZERO,
        }
    }
}

// ── Jetpack ───────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct JetpackState {
    pub fuel: f32,
    pub max_fuel: f32,
    pub force: f32,
    pub fuel_cost_per_sec: f32,
    pub regen_rate: f32,
    pub max_vertical_vel: f32,
    pub is_active: bool,
}

impl Default for JetpackState {
    fn default() -> Self {
        Self {
            fuel: 200.0,
            max_fuel: 200.0,
            force: 0.06,
            fuel_cost_per_sec: 20.0,
            regen_rate: 30.0,
            max_vertical_vel: 0.35,
            is_active: false,
        }
    }
}

// ── Dodge ─────────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone, Default)]
pub struct DodgeState {
    pub is_dodging: bool,
    pub dodge_timer: f32,
    pub cooldown_timer: f32,
    pub dodge_duration: f32,
    pub dodge_cooldown: f32,
    pub dodge_cost: f32,
    pub dodge_speed: f32,
    pub dodge_direction: Vec3,
}

impl DodgeState {
    pub fn new() -> Self {
        Self {
            dodge_duration: 0.3,
            dodge_cooldown: 0.5,
            dodge_cost: 20.0,
            dodge_speed: 1.2,
            ..default()
        }
    }
}

// ── Parry ─────────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone, Default)]
pub struct ParryState {
    pub is_parrying: bool,
    pub parry_timer: f32,
    pub cooldown_timer: f32,
    pub parry_window: f32,
    pub parry_cooldown: f32,
}

impl ParryState {
    pub fn new() -> Self {
        Self {
            parry_window: 0.2,
            parry_cooldown: 1.0,
            ..default()
        }
    }
}

// ── Player State Machine ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Moving,
    Sprinting,
    Dodging,
    Attacking,
    Stunned,
    Dead,
    Jetpack,
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerStateMachine {
    pub current: PlayerState,
    pub previous: Option<PlayerState>,
    pub timer: f32,
}

impl PlayerStateMachine {
    /// Attempt a transition. Returns `true` if the transition was valid.
    pub fn transition(&mut self, next: PlayerState) -> bool {
        use PlayerState::*;
        let allowed = match self.current {
            Idle => matches!(next, Moving | Sprinting | Dodging | Attacking | Stunned | Dead | Jetpack),
            Moving => matches!(next, Idle | Sprinting | Dodging | Attacking | Stunned | Dead | Jetpack),
            Sprinting => matches!(next, Idle | Moving | Dodging | Attacking | Stunned | Dead | Jetpack),
            Dodging => matches!(next, Idle | Moving | Sprinting | Stunned | Dead),
            Attacking => matches!(next, Idle | Moving | Dodging | Stunned | Dead),
            Stunned => matches!(next, Idle | Dead),
            Dead => false,
            Jetpack => matches!(next, Idle | Moving | Stunned | Dead),
        };
        if allowed {
            self.previous = Some(self.current);
            self.current = next;
            self.timer = 0.0;
        }
        allowed
    }

    /// Force-transition regardless of state rules (e.g. death, reset).
    pub fn force(&mut self, next: PlayerState) {
        self.previous = Some(self.current);
        self.current = next;
        self.timer = 0.0;
    }
}

// ── Pitch tracker (on camera child) ──────────────────────────────────────────
#[derive(Component, Default)]
pub struct CameraPitch(pub f32);

// ── Invulnerability flash timer ────────────────────────────────────────────────
#[derive(Component, Default)]
pub struct InvulnerabilityFlash {
    pub timer: f32,
    pub duration: f32,
}
