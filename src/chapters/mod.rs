//! Heavy Water chapter system. Replaces the old wave-based difficulty loop.
//!
//! A chapter is a hand-authored sequence of `EncounterStep`s. The
//! `chapter_plugin` advances through the steps when their completion
//! condition fires (group cleared, dialogue timer elapsed, boss dead).
//!
//! All 14 chapters are stub-playable; Chapter 1 (`Amp!`) is the polished
//! reference. Each chapter sets a biome (recoloring the existing world
//! geometry) and seeds discoverables / companion recruits.

use bevy::prelude::*;

use crate::components::enemy::EnemyType;
use crate::components::faction::Faction;
use crate::components::discoverable::DiscoverableKind;

// ── Chapter ID ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChapterId(pub u8);

impl ChapterId {
    pub const FIRST: Self = ChapterId(1);
    pub const LAST: Self = ChapterId(14);
    pub fn next(self) -> Option<Self> {
        if self.0 < Self::LAST.0 { Some(ChapterId(self.0 + 1)) } else { None }
    }
    pub fn index(self) -> usize { self.0 as usize - 1 }
}

// ── Biome ─────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    RuinedCity,        // Ch.1
    InsectoidMetropolis, // Ch.2
    LunarLabsRuins,    // Ch.3
    BrothersFortress,  // Ch.4
    StarCity,          // Ch.5
    CharsDomain,       // Ch.6
    SwarmHive,         // Ch.7
    Wasteland,         // Ch.8
    PeacefulSanctuary, // Ch.9
    Ashur,             // Ch.10
    EarthVillage,      // Ch.11
    Battleground,      // Ch.12
    WinterField,       // Ch.13
    StarfallArena,     // Ch.14
}

impl Biome {
    /// Returns (sky, fog, ground, accent) palette for this biome.
    pub fn palette(&self) -> (Color, Color, Color, Color) {
        use Biome::*;
        match self {
            RuinedCity        => (c(0.18,0.10,0.06), c(0.10,0.06,0.04), c(0.30,0.25,0.22), c(1.0,0.4,0.1)),
            InsectoidMetropolis=>(c(0.05,0.20,0.05), c(0.04,0.10,0.04), c(0.18,0.25,0.10), c(0.4,1.0,0.2)),
            LunarLabsRuins    => (c(0.10,0.10,0.18), c(0.05,0.05,0.10), c(0.30,0.30,0.40), c(0.4,0.85,1.0)),
            BrothersFortress  => (c(0.15,0.05,0.10), c(0.08,0.03,0.05), c(0.25,0.20,0.20), c(1.0,0.5,0.0)),
            StarCity          => (c(0.04,0.06,0.18), c(0.02,0.04,0.10), c(0.20,0.25,0.45), c(0.0,1.0,1.0)),
            CharsDomain       => (c(0.30,0.06,0.02), c(0.18,0.04,0.01), c(0.40,0.15,0.05), c(1.0,0.2,0.0)),
            SwarmHive         => (c(0.10,0.02,0.15), c(0.05,0.01,0.08), c(0.20,0.05,0.25), c(0.9,0.0,0.6)),
            Wasteland         => (c(0.25,0.20,0.15), c(0.18,0.14,0.10), c(0.45,0.40,0.30), c(0.8,0.6,0.2)),
            PeacefulSanctuary => (c(0.40,0.55,0.65), c(0.30,0.45,0.55), c(0.30,0.55,0.30), c(1.0,0.95,0.7)),
            Ashur             => (c(0.45,0.30,0.10), c(0.25,0.18,0.06), c(0.50,0.40,0.20), c(1.0,0.7,0.2)),
            EarthVillage      => (c(0.50,0.65,0.85), c(0.40,0.55,0.75), c(0.30,0.55,0.20), c(0.9,0.85,0.5)),
            Battleground      => (c(0.20,0.20,0.20), c(0.10,0.10,0.10), c(0.35,0.30,0.25), c(1.0,0.3,0.1)),
            WinterField       => (c(0.55,0.70,0.85), c(0.75,0.85,0.95), c(0.85,0.90,0.95), c(0.7,0.9,1.0)),
            StarfallArena     => (c(0.02,0.02,0.06), c(0.01,0.01,0.04), c(0.10,0.10,0.20), c(1.0,0.9,0.0)),
        }
    }
}

#[inline] fn c(r: f32, g: f32, b: f32) -> Color { Color::srgb(r,g,b) }

// ── Encounter Steps ───────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum EncounterStep {
    /// Fire one or more radio-chatter lines, then advance after `hold` seconds.
    Dialogue { speaker: &'static str, faction: Faction, line: &'static str, hold: f32 },
    /// Spawn a group of enemies of the given type. Step completes when all are dead.
    SpawnGroup { faction: Faction, enemy_type: EnemyType, count: u32, scale: f32 },
    /// Spawn a named mid-boss (preset name from `robots/presets.rs`).
    MidBoss { preset: &'static str, name: &'static str, faction: Faction, scale: f32 },
    /// Spawn the chapter's main boss with intro line.
    BossFight { preset: &'static str, name: &'static str, faction: Faction, intro_line: &'static str, scale: f32 },
    /// Place a discoverable beacon at a position offset from the player.
    PlaceDiscoverable { kind: DiscoverableKind, label: &'static str, offset: Vec3 },
    /// Final outro line + completion fire.
    Outro { line: &'static str },
}

// ── Chapter Definition ────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ChapterDef {
    pub id: ChapterId,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub biome: Biome,
    pub difficulty_scale: f32,
    pub script: Vec<EncounterStep>,
}

// ── Helpers for building scripts ─────────────────────────────────────────────
fn spawn(faction: Faction, t: EnemyType, count: u32, scale: f32) -> EncounterStep {
    EncounterStep::SpawnGroup { faction, enemy_type: t, count, scale }
}
fn dialogue(speaker: &'static str, faction: Faction, line: &'static str, hold: f32) -> EncounterStep {
    EncounterStep::Dialogue { speaker, faction, line, hold }
}
fn mid_boss(preset: &'static str, name: &'static str, faction: Faction, scale: f32) -> EncounterStep {
    EncounterStep::MidBoss { preset, name, faction, scale }
}
fn boss(preset: &'static str, name: &'static str, faction: Faction, intro_line: &'static str, scale: f32) -> EncounterStep {
    EncounterStep::BossFight { preset, name, faction, intro_line, scale }
}
fn place(kind: DiscoverableKind, label: &'static str, offset: Vec3) -> EncounterStep {
    EncounterStep::PlaceDiscoverable { kind, label, offset }
}
fn outro(line: &'static str) -> EncounterStep {
    EncounterStep::Outro { line }
}

// ── All 14 Chapters ───────────────────────────────────────────────────────────
pub fn all_chapters() -> Vec<ChapterDef> {
    use Faction::*;
    // Note: deliberately NOT `use EnemyType::*;` — `EnemyType::Insectoid`
    // would clash with `Faction::Insectoid`. Only the unambiguous variants
    // are brought into scope.
    use EnemyType::{Drone, Soldier, Heavy, Hybrid};

    vec![
        // ── Chapter 1: Amp! ──────────────────────────────────────────────────
        ChapterDef {
            id: ChapterId(1),
            title: "Amp!",
            subtitle: "Awakening in the dust",
            biome: Biome::RuinedCity,
            difficulty_scale: 1.0,
            script: vec![
                dialogue("Amp", Synthetic, "It is a strange world I live in. Surrounded by dust and rain and wind…", 4.0),
                dialogue("Amp", Synthetic, "Where my brothers and sisters have ended up, I do not know. I just hope they are safe.", 4.0),
                spawn(Insectoid, Drone, 4, 1.0),
                dialogue("Amp", Synthetic, "Formic. I know this is your work.", 2.5),
                spawn(Insectoid, EnemyType::Insectoid, 6, 1.0),
                place(DiscoverableKind::BeamSabreUnlock, "Beam Sabre Core", Vec3::new(8.0, 0.5, 0.0)),
                dialogue("Amp", Synthetic, "Mother's blade. It survived.", 3.0),
                spawn(Insectoid, Soldier, 5, 1.1),
                mid_boss("Punisher", "Insectoid Punisher", Insectoid, 1.4),
                place(DiscoverableKind::Blueprint("motorcycle_blueprint"), "Motorcycle Blueprints", Vec3::new(-10.0, 0.5, 12.0)),
                spawn(Charred, Heavy, 2, 1.2),
                boss("AracnoidQueen", "Aracnoid Queen", Insectoid,
                     "QUEEN: Little doll. The good doctor will be pleased to find you alive.", 2.0),
                place(DiscoverableKind::CompanionRecruit("Aria"), "Aria — Synthetic", Vec3::new(0.0, 0.5, -8.0)),
                place(DiscoverableKind::CompanionRecruit("Valor"), "Valor — Synthetic", Vec3::new(4.0, 0.5, -8.0)),
                outro("Amp: Two of them. Then there is hope."),
            ],
        },

        // ── Chapter 2: The Plague ────────────────────────────────────────────
        ChapterDef {
            id: ChapterId(2),
            title: "The Plague",
            subtitle: "Three years lost",
            biome: Biome::InsectoidMetropolis,
            difficulty_scale: 1.2,
            script: vec![
                dialogue("Amp", Synthetic, "Three years. The Insectoids took everything.", 3.5),
                spawn(Insectoid, Drone, 6, 1.2),
                spawn(Insectoid, EnemyType::Insectoid, 8, 1.2),
                place(DiscoverableKind::WeaponMod("missile_launcher"), "Missile Launcher Mod", Vec3::new(6.0, 0.5, 6.0)),
                dialogue("Lambda", Insectoid, "LAMBDA: …Sister?", 2.0),
                dialogue("Epsilon", Insectoid, "EPSILON: KILL.", 2.0),
                mid_boss("Lambda", "Lambda — Corrupted", Insectoid, 1.5),
                mid_boss("Epsilon", "Epsilon — Corrupted", Insectoid, 1.5),
                place(DiscoverableKind::Blueprint("jet_blueprint"), "Jet Blueprints", Vec3::new(-8.0, 0.5, 4.0)),
                boss("InsectoidGeneral", "Insectoid General", Insectoid,
                     "GENERAL: Formic prepared us for you, Synthetic.", 2.5),
                place(DiscoverableKind::CompanionRecruit("Volt"), "Volt — Synthetic", Vec3::new(0.0, 0.5, -6.0)),
                place(DiscoverableKind::CompanionRecruit("Chroma"), "Chroma — Synthetic", Vec3::new(4.0, 0.5, -6.0)),
                outro("Amp: Volt. Chroma. We need to find the others."),
            ],
        },

        // ── Chapter 3: A Tale of 3 Sisters ───────────────────────────────────
        ChapterDef {
            id: ChapterId(3),
            title: "A Tale of 3 Sisters",
            subtitle: "Lunar Labs, twenty years ago",
            biome: Biome::LunarLabsRuins,
            difficulty_scale: 1.3,
            script: vec![
                dialogue("Cynthia", Civilian, "CYNTHIA: With our minds combined, we will birth Amp first.", 4.0),
                spawn(Insectoid, Soldier, 4, 1.3),
                dialogue("Daria", Synthetic, "DARIA: Sisters. Mother is gone. Run.", 3.0),
                spawn(Insectoid, Heavy, 3, 1.3),
                mid_boss("FormicAvatar", "Formic — Memory Echo", Insectoid, 1.5),
                place(DiscoverableKind::CompanionRecruit("Daria"), "Daria", Vec3::new(0.0, 0.5, -6.0)),
                place(DiscoverableKind::CompanionRecruit("Prima"), "Prima", Vec3::new(4.0, 0.5, -6.0)),
                boss("AracnoidQueen", "Hive Mother", Insectoid,
                     "HIVE MOTHER: All of Cynthia's children belong to Formic now.", 1.5),
                outro("Amp: My sisters. We are reunited."),
            ],
        },

        // ── Chapter 4: A Tale of 4 Brothers ──────────────────────────────────
        ChapterDef {
            id: ChapterId(4),
            title: "A Tale of 4 Brothers",
            subtitle: "Saturn, Mercury, Apollo, Axe",
            biome: Biome::BrothersFortress,
            difficulty_scale: 1.4,
            script: vec![
                dialogue("Apollo", Mechanoid, "APOLLO: Stand with us, Synthetic. The Charred press from the east.", 3.0),
                spawn(Charred, Soldier, 6, 1.4),
                spawn(Charred, Heavy, 3, 1.4),
                mid_boss("CharredCaptain", "Charred Captain", Charred, 1.5),
                place(DiscoverableKind::WeaponMod("piercing_rounds"), "Piercing Rounds", Vec3::new(8.0, 0.5, 0.0)),
                boss("HarvesterMech", "Char Harvester Mech", Charred,
                     "HARVESTER: COMPLIANCE. OR INCINERATION.", 2.0),
                place(DiscoverableKind::CompanionRecruit("Apollo"), "Apollo — Mechanoid", Vec3::new(0.0, 0.5, -6.0)),
                outro("Apollo: My brothers and I owe you a debt, Synthetic."),
            ],
        },

        // ── Chapter 5: StarCity Soldier ──────────────────────────────────────
        ChapterDef {
            id: ChapterId(5),
            title: "StarCity Soldier",
            subtitle: "Sergio Wolfrim's last stand",
            biome: Biome::StarCity,
            difficulty_scale: 1.5,
            script: vec![
                dialogue("Sergio", Civilian, "SERGIO: Amp. The Star whispers your name.", 3.0),
                spawn(Insectoid, Soldier, 5, 1.5),
                spawn(Charred, Heavy, 3, 1.5),
                mid_boss("Octavius", "Octavius", Mechanoid, 1.5),
                place(DiscoverableKind::ArmorMod("reactive_plating"), "Reactive Plating", Vec3::new(0.0, 0.5, 8.0)),
                boss("BruteForge", "Star City Sentinel", Mechanoid,
                     "SENTINEL: Prove you are worthy of the Star.", 2.0),
                place(DiscoverableKind::CompanionRecruit("Atlas"), "Atlas — Synthetic", Vec3::new(0.0, 0.5, -6.0)),
                outro("Sergio: The Mechanoids stand with you now."),
            ],
        },

        // ── Chapter 6: Char's Domain ─────────────────────────────────────────
        ChapterDef {
            id: ChapterId(6),
            title: "Char's Domain",
            subtitle: "Scorched earth",
            biome: Biome::CharsDomain,
            difficulty_scale: 1.6,
            script: vec![
                dialogue("Amp", Synthetic, "Char's flames burn even the ash.", 2.5),
                spawn(Charred, Soldier, 8, 1.6),
                spawn(Animaton, Heavy, 4, 1.6),
                mid_boss("WolfAnimaton", "Wolf Animaton", Animaton, 1.6),
                mid_boss("TigerAnimaton", "Tiger Animaton", Animaton, 1.6),
                place(DiscoverableKind::ArmorMod("coolant_weave"), "Coolant Weave", Vec3::new(6.0, 0.5, 6.0)),
                boss("HarvesterMech", "Char — Final Form", Charred,
                     "CHAR: I AM THE FIRE THAT MADE THIS WORLD.", 2.5),
                outro("Amp: Char is broken. The eastern wastes are quiet."),
            ],
        },

        // ── Chapter 7: The Swarm ─────────────────────────────────────────────
        ChapterDef {
            id: ChapterId(7),
            title: "The Swarm",
            subtitle: "A greater evil from the shadows",
            biome: Biome::SwarmHive,
            difficulty_scale: 1.8,
            script: vec![
                dialogue("Cygnus", Swarm, "CYGNUS: Worlds end. We are the ending.", 3.0),
                spawn(Swarm, Drone, 10, 1.8),
                spawn(Swarm, Soldier, 6, 1.8),
                mid_boss("Brutus", "Brutus", Swarm, 1.7),
                mid_boss("Nero", "Nero", Swarm, 1.7),
                place(DiscoverableKind::WeaponMod("piercing_rounds"), "Swarm Piercer", Vec3::new(0.0, 0.5, 10.0)),
                boss("Cygnus", "King Cygnus", Swarm,
                     "CYGNUS: This world tastes like the last one.", 2.0),
                outro("Amp: The King falls. The Queen will come for vengeance."),
            ],
        },

        // ── Chapter 8: Ruination ─────────────────────────────────────────────
        ChapterDef {
            id: ChapterId(8),
            title: "Ruination",
            subtitle: "Earth abandoned",
            biome: Biome::Wasteland,
            difficulty_scale: 1.9,
            script: vec![
                dialogue("Amp", Synthetic, "Nothing left. Nothing but us.", 3.0),
                spawn(Swarm, Heavy, 5, 1.9),
                spawn(Insectoid, EnemyType::Insectoid, 6, 1.9),
                mid_boss("Minerva", "Minerva", Swarm, 1.8),
                boss("Cygni", "Queen Cygni", Swarm,
                     "CYGNI: I will eat your stars one by one.", 2.5),
                outro("Amp: The Swarm Queen falls. We must rebuild."),
            ],
        },

        // ── Chapter 9: Peaceful DNA ──────────────────────────────────────────
        ChapterDef {
            id: ChapterId(9),
            title: "Peaceful DNA",
            subtitle: "Theta's gift",
            biome: Biome::PeacefulSanctuary,
            difficulty_scale: 1.5,
            script: vec![
                dialogue("Theta", Synthetic, "THETA: Brother. Free them. Free them all.", 3.0),
                spawn(Animaton, Drone, 6, 1.4),
                place(DiscoverableKind::CompanionRecruit("Theta"), "Theta — Synthetic", Vec3::new(0.0, 0.5, -4.0)),
                place(DiscoverableKind::CompanionRecruit("Ion"), "Ion — Mechanoid", Vec3::new(4.0, 0.5, -4.0)),
                boss("HarvesterMech", "Last Animaton", Animaton,
                     "ANIMATON: …let me … rest …", 1.6),
                outro("Theta: A new soul is born. His name is Ion."),
            ],
        },

        // ── Chapter 10: The Land of Ashur ────────────────────────────────────
        ChapterDef {
            id: ChapterId(10),
            title: "The Land of Ashur",
            subtitle: "Forgotten kingdom",
            biome: Biome::Ashur,
            difficulty_scale: 1.9,
            script: vec![
                dialogue("Aruna", Mechanoid, "ARUNA: The desert remembers what humans forgot.", 3.0),
                spawn(Swarm, Drone, 8, 1.9),
                mid_boss("Helios", "Helios", Mechanoid, 1.7),
                boss("BruteForge", "Ashur Sentinel", Mechanoid, "SENTINEL: Pass, or perish.", 2.0),
                outro("Amp: Ashur kneels. The road north is open."),
            ],
        },

        // ── Chapter 11: A Village Called Earth ───────────────────────────────
        ChapterDef {
            id: ChapterId(11),
            title: "A Village Called Earth",
            subtitle: "What humanity became",
            biome: Biome::EarthVillage,
            difficulty_scale: 2.0,
            script: vec![
                dialogue("Villager", Civilian, "VILLAGER: Strangers from the metal world!", 3.0),
                spawn(Swarm, Soldier, 8, 2.0),
                mid_boss("Caliguon", "Caliguon", Swarm, 1.9),
                boss("Cygnus", "Swarm Vanguard", Swarm,
                     "VANGUARD: The Queen sends her regards.", 2.0),
                outro("Amp: The villagers live. That alone is victory."),
            ],
        },

        // ── Chapter 12: Evolution or Eradication ─────────────────────────────
        ChapterDef {
            id: ChapterId(12),
            title: "Evolution or Eradication",
            subtitle: "The choice",
            biome: Biome::Battleground,
            difficulty_scale: 2.2,
            script: vec![
                dialogue("Amp", Synthetic, "Choose. Adapt or die.", 3.0),
                spawn(Swarm, Heavy, 6, 2.2),
                spawn(Insectoid, Hybrid, 2, 2.2),
                mid_boss("HybridOmega", "Hybrid Omega", Insectoid, 2.0),
                boss("Cygni", "Cygni — Awakened", Swarm,
                     "CYGNI: I will not fall again.", 2.5),
                outro("Amp: We chose. We endured."),
            ],
        },

        // ── Chapter 13: A Calm Winter Night ──────────────────────────────────
        ChapterDef {
            id: ChapterId(13),
            title: "A Calm Winter Night",
            subtitle: "Before the end",
            biome: Biome::WinterField,
            difficulty_scale: 1.8,
            script: vec![
                dialogue("Daria", Synthetic, "DARIA: Snow on metal. It is almost beautiful.", 4.0),
                dialogue("Amp", Synthetic, "Listen. The sky is too quiet.", 3.0),
                spawn(Swarm, Drone, 6, 1.8),
                mid_boss("Selene", "Selene", Mechanoid, 1.8),
                boss("HybridOmega", "Winter Stalker", Swarm, "STALKER: …", 1.9),
                outro("Amp: Stars are falling. Tomorrow we end this."),
            ],
        },

        // ── Chapter 14: Starfall ─────────────────────────────────────────────
        ChapterDef {
            id: ChapterId(14),
            title: "Starfall",
            subtitle: "The Star, the Swarm, and the Six",
            biome: Biome::StarfallArena,
            difficulty_scale: 2.5,
            script: vec![
                dialogue("Amp", Synthetic, "All of us. Together. For the last time.", 3.0),
                spawn(Swarm, Hybrid, 4, 2.5),
                mid_boss("Cygnus", "Cygnus — Reborn", Swarm, 2.2),
                mid_boss("Cygni", "Cygni — Reborn", Swarm, 2.2),
                dialogue("Amp", Synthetic, "Six unknown signatures. From the future?", 3.0),
                boss("HybridOmega", "Starfall — Final Form", Swarm,
                     "VOICE: I am every ending. I am every beginning.", 3.0),
                outro("Amp: The Star is silent. The world is ours to remake."),
            ],
        },
    ]
}

pub fn get_chapter(id: ChapterId) -> Option<ChapterDef> {
    all_chapters().into_iter().find(|c| c.id == id)
}
