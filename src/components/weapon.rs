use bevy::prelude::*;

// ── Weapon Type ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Pistol,    // 1
    Rifle,     // 2
    Shotgun,   // 3
    Rocket,    // 4
    Laser,     // 5
    Grenade,   // 6
}

impl WeaponType {
    pub fn display_name(&self) -> &'static str {
        match self {
            WeaponType::Pistol => "Plasma Pistol",
            WeaponType::Rifle => "Pulse Rifle",
            WeaponType::Shotgun => "Scatter Blaster",
            WeaponType::Rocket => "Nova Launcher",
            WeaponType::Laser => "Photon Beam",
            WeaponType::Grenade => "Fusion Grenades",
        }
    }
}

// ── Primary Weapon Component ──────────────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub fire_rate: f32,    // seconds between shots
    pub ammo: u32,
    pub max_ammo: u32,
    pub speed: f32,         // projectile speed (units/sec)
    pub spread: f32,
    pub automatic: bool,
    pub fire_timer: f32,
    pub pellets: u32,       // for shotgun
    pub is_explosive: bool,
    pub explosion_radius: f32,
}

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        match weapon_type {
            WeaponType::Pistol => Self {
                weapon_type, damage: 15.0, fire_rate: 0.3,
                ammo: 50, max_ammo: 50, speed: 60.0,
                spread: 0.02, automatic: false, fire_timer: 0.0,
                pellets: 1, is_explosive: false, explosion_radius: 0.0,
            },
            WeaponType::Rifle => Self {
                weapon_type, damage: 25.0, fire_rate: 0.1,
                ammo: 120, max_ammo: 120, speed: 90.0,
                spread: 0.03, automatic: true, fire_timer: 0.0,
                pellets: 1, is_explosive: false, explosion_radius: 0.0,
            },
            WeaponType::Shotgun => Self {
                weapon_type, damage: 8.0, fire_rate: 0.8,
                ammo: 24, max_ammo: 24, speed: 75.0,
                spread: 0.15, automatic: false, fire_timer: 0.0,
                pellets: 8, is_explosive: false, explosion_radius: 0.0,
            },
            WeaponType::Rocket => Self {
                weapon_type, damage: 100.0, fire_rate: 1.5,
                ammo: 8, max_ammo: 8, speed: 30.0,
                spread: 0.0, automatic: false, fire_timer: 0.0,
                pellets: 1, is_explosive: true, explosion_radius: 6.0,
            },
            WeaponType::Laser => Self {
                weapon_type, damage: 40.0, fire_rate: 0.05,
                ammo: 200, max_ammo: 200, speed: 300.0,
                spread: 0.0, automatic: true, fire_timer: 0.0,
                pellets: 1, is_explosive: false, explosion_radius: 0.0,
            },
            WeaponType::Grenade => Self {
                weapon_type, damage: 80.0, fire_rate: 1.0,
                ammo: 6, max_ammo: 6, speed: 15.0,
                spread: 0.0, automatic: false, fire_timer: 0.0,
                pellets: 1, is_explosive: true, explosion_radius: 8.0,
            },
        }
    }

    pub fn can_fire(&self) -> bool {
        self.fire_timer <= 0.0 && self.ammo > 0
    }

    pub fn reload(&mut self) {
        self.ammo = self.max_ammo;
    }
}

// ── Active Weapon Tracker (on Player) ────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct WeaponInventory {
    pub slots: [Weapon; 6],
    pub active_slot: usize, // 0-5 = keys 1-6
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            slots: [
                Weapon::new(WeaponType::Pistol),
                Weapon::new(WeaponType::Rifle),
                Weapon::new(WeaponType::Shotgun),
                Weapon::new(WeaponType::Rocket),
                Weapon::new(WeaponType::Laser),
                Weapon::new(WeaponType::Grenade),
            ],
            active_slot: 0,
        }
    }
}

impl WeaponInventory {
    pub fn active(&self) -> &Weapon {
        &self.slots[self.active_slot]
    }

    pub fn active_mut(&mut self) -> &mut Weapon {
        &mut self.slots[self.active_slot]
    }
}

// ── Special Weapon Slot ───────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialSlot {
    Slot7, // Homing Missile
    Slot8, // Tracking Energy Burst
    Slot9, // Bomb
    Slot0, // Combat Drone
}

#[derive(Component, Debug, Clone)]
pub struct SpecialWeapon {
    pub slot: SpecialSlot,
    pub name: &'static str,
    pub base_damage: f32,
    pub cooldown: f32,
    pub cooldown_timer: f32,
    pub ammo: u32,
    pub max_ammo: u32,
    pub level: u32,
}

impl SpecialWeapon {
    pub fn new(slot: SpecialSlot) -> Self {
        match slot {
            SpecialSlot::Slot7 => Self {
                slot, name: "Homing Missile",
                base_damage: 60.0, cooldown: 2.0, cooldown_timer: 0.0,
                ammo: 10, max_ammo: 10, level: 1,
            },
            SpecialSlot::Slot8 => Self {
                slot, name: "Energy Burst",
                base_damage: 45.0, cooldown: 1.5, cooldown_timer: 0.0,
                ammo: 15, max_ammo: 15, level: 1,
            },
            SpecialSlot::Slot9 => Self {
                slot, name: "Bomb",
                base_damage: 120.0, cooldown: 4.0, cooldown_timer: 0.0,
                ammo: 5, max_ammo: 5, level: 1,
            },
            SpecialSlot::Slot0 => Self {
                slot, name: "Combat Drone",
                base_damage: 20.0, cooldown: 10.0, cooldown_timer: 0.0,
                ammo: 3, max_ammo: 3, level: 1,
            },
        }
    }

    pub fn can_fire(&self) -> bool {
        self.cooldown_timer <= 0.0 && self.ammo > 0
    }

    pub fn effective_damage(&self) -> f32 {
        let mult = match self.level {
            1 => 1.0,
            2 => 1.4,
            _ => 1.7,
        };
        self.base_damage * mult
    }
}

// ── Special Weapon Inventory (on Player) ─────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct SpecialWeaponInventory {
    pub slot7: SpecialWeapon,
    pub slot8: SpecialWeapon,
    pub slot9: SpecialWeapon,
    pub slot0: SpecialWeapon,
}

impl Default for SpecialWeaponInventory {
    fn default() -> Self {
        Self {
            slot7: SpecialWeapon::new(SpecialSlot::Slot7),
            slot8: SpecialWeapon::new(SpecialSlot::Slot8),
            slot9: SpecialWeapon::new(SpecialSlot::Slot9),
            slot0: SpecialWeapon::new(SpecialSlot::Slot0),
        }
    }
}

// ── Beam Sabre ────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct BeamSabre {
    pub active: bool,
    /// Heavy Water: Beam Sabre is locked until the Ch.1 discoverable is collected.
    pub unlocked: bool,
    pub level: u32,
    pub slash_damage: f32,
    pub wave_damage: f32,
    pub slash_count: u32,
    pub cooldown: f32,
    pub cooldown_timer: f32,
    pub slash_timer: f32,
    pub slash_index: u32,
    pub is_slashing: bool,
}

/// Marker on the player while the Beam Sabre has not been discovered yet.
#[derive(Component, Debug, Default)]
pub struct BeamSabreLocked;

impl Default for BeamSabre {
    fn default() -> Self {
        Self {
            active: false,
            unlocked: false,
            level: 1,
            slash_damage: 25.0,
            wave_damage: 40.0,
            slash_count: 2,
            cooldown: 0.8,
            cooldown_timer: 0.0,
            slash_timer: 0.0,
            slash_index: 0,
            is_slashing: false,
        }
    }
}

impl BeamSabre {
    pub fn set_level(&mut self, level: u32) {
        self.level = level;
        let (sd, wd, sc, cd) = match level {
            1 => (25.0, 40.0, 2, 0.8),
            2 => (35.0, 60.0, 2, 0.7),
            3 => (50.0, 80.0, 3, 0.6),
            4 => (65.0, 100.0, 4, 0.5),
            _ => (85.0, 150.0, 5, 0.4),
        };
        self.slash_damage = sd;
        self.wave_damage = wd;
        self.slash_count = sc;
        self.cooldown = cd;
    }

    pub fn is_piercing(&self) -> bool { self.level >= 3 }
    pub fn fires_dual_wave(&self) -> bool { self.level >= 4 }
    pub fn has_aoe_splash(&self) -> bool { self.level >= 5 }
}

// ── Projectile ────────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub direction: Vec3,
    pub lifetime: f32,
    pub is_explosive: bool,
    pub explosion_radius: f32,
    pub weapon_type: ProjectileOwner,
    pub owner: Option<Entity>,
    pub piercing: bool,
    /// For grenade arc
    pub gravity_affected: bool,
    pub vertical_velocity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
    Companion,
    Drone,
    Missile,
    EnergyBurst,
    Bomb,
}

// ── Melee Combo ───────────────────────────────────────────────────────────────
#[derive(Component, Debug, Clone, Default)]
pub struct MeleeCombo {
    pub light_index: usize,
    pub heavy_index: usize,
    pub light_timer: f32,
    pub heavy_timer: f32,
    pub active_timer: f32,
    pub is_attacking: bool,
    pub buffered_light: bool,
    pub buffered_heavy: bool,
    pub damage_multiplier: f32,
}

impl MeleeCombo {
    pub fn new() -> Self {
        Self {
            damage_multiplier: 1.0,
            ..default()
        }
    }
}
