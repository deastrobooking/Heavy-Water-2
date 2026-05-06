use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// Story factions. Drives enemy color/preset choice, dialogue, and recruitment rules.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Faction {
    /// Synthetics — Cynthia You's humanoid line (Amp, Atlas, Volt, Chroma, Daria, Prima, Theta, Ion, Valor).
    Synthetic,
    /// Mechanoids — Ancient race forged by Sergio Wolfrim from the Star meteor.
    Mechanoid,
    /// Swarm — galactic mechanoid empire ruled by Cygnus & Cygni.
    Swarm,
    /// Insectoids — Dr. Formic's stolen tech bred into chitinous synth-insects.
    Insectoid,
    /// Animatons — Char's animal-DNA mech hybrids built from a captured Mechanoid.
    Animaton,
    /// Char's domain forces (human-aligned, scorched-earth tech).
    Charred,
    /// Civilians (Earth villagers, Star City inhabitants, etc.).
    Civilian,
    /// Default / unaffiliated.
    #[default]
    Neutral,
}

impl Faction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Faction::Synthetic => "Synthetic",
            Faction::Mechanoid => "Mechanoid",
            Faction::Swarm => "Swarm",
            Faction::Insectoid => "Insectoid",
            Faction::Animaton => "Animaton",
            Faction::Charred => "Charred",
            Faction::Civilian => "Civilian",
            Faction::Neutral => "Neutral",
        }
    }

    /// Voice/portrait tint used by the radio-chatter HUD.
    pub fn dialogue_color(&self) -> Color {
        match self {
            Faction::Synthetic => Color::srgb(0.4, 0.85, 1.0),
            Faction::Mechanoid => Color::srgb(1.0, 0.75, 0.2),
            Faction::Swarm => Color::srgb(0.9, 0.1, 0.3),
            Faction::Insectoid => Color::srgb(0.4, 1.0, 0.2),
            Faction::Animaton => Color::srgb(1.0, 0.4, 0.1),
            Faction::Charred => Color::srgb(0.7, 0.25, 0.0),
            Faction::Civilian => Color::srgb(0.85, 0.85, 0.85),
            Faction::Neutral => Color::srgb(0.7, 0.7, 0.8),
        }
    }
}

/// Component tag for any entity with a story name (boss, recruitable, key NPC).
#[derive(Component, Debug, Clone)]
pub struct NamedCharacter {
    pub id: &'static str,
    pub display_name: &'static str,
    pub faction: Faction,
}
