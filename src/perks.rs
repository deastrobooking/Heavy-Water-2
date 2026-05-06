//! Perk tree — Synthetic / Mechanoid / Stealth branches.
//! Each level-up grants 1 unspent point; spend in chapter-select UI.

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerkBranch {
    Synthetic, // HP / healing
    Mechanoid, // Damage
    Stealth,   // Dodge / parry / stamina
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerkDef {
    pub id: &'static str,
    pub name: &'static str,
    pub branch: PerkBranch,
    pub description: &'static str,
    pub max_rank: u32,
}

pub fn all_perks() -> Vec<PerkDef> {
    vec![
        PerkDef { id: "synth_vitality",  name: "Synthetic Vitality",  branch: PerkBranch::Synthetic, description: "+15 max HP per rank.",         max_rank: 5 },
        PerkDef { id: "synth_regen",     name: "Self-Repair Routine", branch: PerkBranch::Synthetic, description: "+0.5 HP/sec out of combat.",   max_rank: 4 },
        PerkDef { id: "mech_overcharge",name: "Overcharge",           branch: PerkBranch::Mechanoid, description: "+5% weapon damage per rank.",  max_rank: 5 },
        PerkDef { id: "mech_munitions", name: "Extended Munitions",   branch: PerkBranch::Mechanoid, description: "+15% max ammo per rank.",      max_rank: 3 },
        PerkDef { id: "stealth_evasion",name: "Evasion Routines",     branch: PerkBranch::Stealth,   description: "-10% dodge stamina cost.",      max_rank: 3 },
        PerkDef { id: "stealth_parry",  name: "Predictive Parry",     branch: PerkBranch::Stealth,   description: "+0.05s parry window per rank.", max_rank: 3 },
    ]
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerkTree {
    pub points_unspent: u32,
    pub ranks: Vec<(String, u32)>, // (perk id, rank)
}

impl PerkTree {
    pub fn rank(&self, perk_id: &str) -> u32 {
        self.ranks.iter().find(|(id, _)| id == perk_id).map(|(_, r)| *r).unwrap_or(0)
    }
    pub fn try_spend(&mut self, perk_id: &str) -> bool {
        if self.points_unspent == 0 { return false; }
        let perks = all_perks();
        let Some(def) = perks.iter().find(|p| p.id == perk_id) else { return false };
        let cur = self.rank(perk_id);
        if cur >= def.max_rank { return false; }
        if let Some(entry) = self.ranks.iter_mut().find(|(id, _)| id == perk_id) {
            entry.1 += 1;
        } else {
            self.ranks.push((perk_id.to_string(), 1));
        }
        self.points_unspent -= 1;
        true
    }
    pub fn award(&mut self, points: u32) {
        self.points_unspent += points;
    }
    pub fn damage_mult(&self) -> f32 { 1.0 + 0.05 * self.rank("mech_overcharge") as f32 }
    pub fn ammo_mult(&self) -> f32   { 1.0 + 0.15 * self.rank("mech_munitions") as f32 }
    pub fn hp_bonus(&self) -> f32    { 15.0 * self.rank("synth_vitality") as f32 }
    pub fn regen_per_sec(&self) -> f32 { 0.5 * self.rank("synth_regen") as f32 }
    pub fn dodge_cost_mult(&self) -> f32 { 1.0 - 0.10 * self.rank("stealth_evasion") as f32 }
    pub fn parry_window_bonus(&self) -> f32 { 0.05 * self.rank("stealth_parry") as f32 }
}
