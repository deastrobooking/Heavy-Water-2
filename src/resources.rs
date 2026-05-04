use bevy::prelude::*;

// ── Wave State ────────────────────────────────────────────────────────────────
#[derive(Resource, Debug, Default)]
pub struct WaveInfo {
    pub wave_number: u32,
    pub wave_timer: f32,
    pub wave_duration: f32,  // seconds per wave (default 60)
    pub enemy_count: u32,
    pub max_enemies: u32,
    pub spawn_timer: f32,
    pub spawn_interval: f32, // seconds between spawns
}

impl WaveInfo {
    pub fn new() -> Self {
        Self {
            wave_number: 1,
            wave_timer: 0.0,
            wave_duration: 60.0,
            enemy_count: 0,
            max_enemies: 20,
            spawn_timer: 0.0,
            spawn_interval: 5.0,
        }
    }

    /// Advance to the next wave, scaling difficulty.
    pub fn advance(&mut self) {
        self.wave_number += 1;
        self.wave_timer = 0.0;
        // Increase cap by 2 per wave, up to 50
        self.max_enemies = (self.max_enemies + 2).min(50);
        // Decrease spawn interval by 0.2s per wave, down to 2s
        self.spawn_interval = (self.spawn_interval - 0.2).max(2.0);
    }

    /// Enemy stat scaling multiplier for the current wave.
    pub fn difficulty_multiplier(&self) -> f32 {
        1.0 + (self.wave_number.saturating_sub(1) as f32) * 0.2
    }
}

// ── Game Settings ─────────────────────────────────────────────────────────────
#[derive(Resource, Debug)]
pub struct GameSettings {
    pub mouse_sensitivity: f32,
    pub master_volume: f32,
    pub show_damage_numbers: bool,
    pub world_seed: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.0008,
            master_volume: 1.0,
            show_damage_numbers: true,
            world_seed: 42_195,
        }
    }
}

// ── UI Message Queue ──────────────────────────────────────────────────────────
#[derive(Resource, Debug, Default)]
pub struct UiMessage {
    pub text: String,
    pub timer: f32,
}

// ── Player Score ──────────────────────────────────────────────────────────────
#[derive(Resource, Debug, Default)]
pub struct PlayerScore {
    pub kills: u32,
    pub total_damage_dealt: f32,
    pub chests_opened: u32,
    pub waves_survived: u32,
}
