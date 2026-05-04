use bevy::prelude::*;
use super::designer::{RobotStyle, RobotArchetype, HeadShape, LegStyle, VisorStyle, ArmStyle};

// ── Enemy Presets ─────────────────────────────────────────────────────────────

pub fn scout_prime() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Scout,
        scale: 1.0,
        torso_width: 18.0, torso_height: 36.0, torso_depth: 13.0,
        head_size: 10.0, head_shape: HeadShape::Box,
        arm_length: 26.0, arm_thickness: 5.0, arm_style: ArmStyle::Box,
        leg_length: 28.0, leg_thickness: 7.0, leg_style: LegStyle::Box,
        shoulder_pad_size: 6.0, hip_pad_size: 5.0,
        has_visor: true, visor_style: VisorStyle::Slit,
        has_cannons: false,
        primary: Color::srgb(0.2, 0.4, 0.8),
        secondary: Color::srgb(0.1, 0.2, 0.4),
        emissive: Color::srgb(0.0, 0.6, 1.0),
        ..RobotStyle::default()
    }
}

pub fn brute_forge() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Brute,
        scale: 1.1,
        torso_width: 32.0, torso_height: 50.0, torso_depth: 22.0,
        head_size: 13.0, head_shape: HeadShape::Box,
        arm_length: 32.0, arm_thickness: 11.0, arm_style: ArmStyle::Box,
        leg_length: 32.0, leg_thickness: 13.0, leg_style: LegStyle::Box,
        shoulder_pad_size: 12.0, hip_pad_size: 8.0,
        has_visor: true, visor_style: VisorStyle::Slit,
        has_cannons: true, cannon_size: 5.0,
        has_horns: true, horn_length: 14.0,
        extra_plating: 2, asymmetry: 0.2,
        primary: Color::srgb(0.7, 0.15, 0.05),
        secondary: Color::srgb(0.35, 0.07, 0.02),
        emissive: Color::srgb(1.0, 0.4, 0.0),
        ..RobotStyle::default()
    }
}

pub fn jet_warden() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Flyer,
        scale: 1.0,
        torso_width: 16.0, torso_height: 34.0, torso_depth: 12.0,
        head_size: 9.0, head_shape: HeadShape::Sphere,
        arm_length: 26.0, arm_thickness: 5.0, arm_style: ArmStyle::Cylinder,
        leg_length: 22.0, leg_thickness: 6.0, leg_style: LegStyle::Hoverpads,
        shoulder_pad_size: 7.0, hip_pad_size: 4.0,
        has_wings: true, wing_span: 40.0, wing_angle: 35.0,
        has_visor: true, visor_style: VisorStyle::Full,
        has_backpack: true, backpack_size: 12.0,
        primary: Color::srgb(0.2, 0.1, 0.4),
        secondary: Color::srgb(0.1, 0.05, 0.2),
        emissive: Color::srgb(0.6, 0.0, 1.0),
        ..RobotStyle::default()
    }
}

pub fn tank_titan() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Tank,
        scale: 1.2,
        torso_width: 38.0, torso_height: 56.0, torso_depth: 28.0,
        head_size: 14.0, head_shape: HeadShape::Box,
        arm_length: 34.0, arm_thickness: 13.0, arm_style: ArmStyle::Box,
        leg_length: 36.0, leg_thickness: 15.0, leg_style: LegStyle::Box,
        shoulder_pad_size: 15.0, hip_pad_size: 10.0,
        has_visor: true, visor_style: VisorStyle::Slit,
        has_shield: true, shield_size: 40.0,
        has_horns: true, horn_length: 18.0,
        extra_plating: 3, asymmetry: 0.1,
        primary: Color::srgb(0.1, 0.3, 0.1),
        secondary: Color::srgb(0.05, 0.15, 0.05),
        emissive: Color::srgb(0.0, 1.0, 0.3),
        ..RobotStyle::default()
    }
}

pub fn insectoid_stalker() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Insectoid,
        scale: 0.9,
        torso_width: 16.0, torso_height: 32.0, torso_depth: 11.0,
        head_size: 9.0, head_shape: HeadShape::Sphere,
        arm_length: 30.0, arm_thickness: 5.0, arm_style: ArmStyle::Tapered,
        leg_length: 34.0, leg_thickness: 7.0, leg_style: LegStyle::Digitigrade,
        shoulder_pad_size: 5.0, hip_pad_size: 4.0,
        has_visor: true, visor_style: VisorStyle::Full,
        has_wings: true, wing_span: 35.0, wing_angle: 50.0,
        has_tail: true, tail_length: 28.0, tail_segments: 5,
        has_antennae: true, antenna_length: 22.0,
        primary: Color::srgb(0.1, 0.35, 0.05),
        secondary: Color::srgb(0.05, 0.18, 0.02),
        emissive: Color::srgb(0.2, 0.9, 0.0),
        ..RobotStyle::default()
    }
}

pub fn hybrid_omega() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Hybrid,
        scale: 1.4,
        torso_width: 28.0, torso_height: 50.0, torso_depth: 20.0,
        head_size: 14.0, head_shape: HeadShape::Cone,
        arm_length: 34.0, arm_thickness: 9.0, arm_style: ArmStyle::Box,
        leg_length: 36.0, leg_thickness: 11.0, leg_style: LegStyle::Digitigrade,
        shoulder_pad_size: 12.0, hip_pad_size: 8.0,
        has_wings: true, wing_span: 55.0, wing_angle: 40.0,
        has_tail: true, tail_length: 35.0, tail_segments: 6,
        has_shield: true, shield_size: 44.0,
        has_cannons: true, cannon_size: 7.0,
        has_horns: true, horn_length: 20.0,
        has_antennae: true, antenna_length: 18.0,
        has_visor: true, visor_style: VisorStyle::Full,
        extra_plating: 2, asymmetry: 0.3,
        primary: Color::srgb(0.25, 0.0, 0.35),
        secondary: Color::srgb(0.12, 0.0, 0.18),
        emissive: Color::srgb(0.9, 0.0, 1.0),
        ..RobotStyle::default()
    }
}

// ── Ally Presets ──────────────────────────────────────────────────────────────

pub fn guardian_unit() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Ally,
        scale: 1.0,
        torso_width: 20.0, torso_height: 40.0, torso_depth: 15.0,
        has_shield: true, shield_size: 32.0,
        has_cannons: true, cannon_size: 4.0,
        has_backpack: true, backpack_size: 11.0,
        has_visor: true, visor_style: VisorStyle::Slit,
        primary: Color::srgb(0.1, 0.6, 0.7),
        secondary: Color::srgb(0.05, 0.3, 0.35),
        emissive: Color::srgb(0.0, 1.0, 1.0),
        ..RobotStyle::default()
    }
}

pub fn medic_drone_style() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Ally,
        scale: 0.8,
        torso_width: 14.0, torso_height: 28.0, torso_depth: 11.0,
        leg_style: LegStyle::Hoverpads,
        has_antennae: true, antenna_length: 18.0,
        has_backpack: true, backpack_size: 8.0,
        has_visor: true, visor_style: VisorStyle::Round,
        primary: Color::srgb(0.1, 0.55, 0.2),
        secondary: Color::srgb(0.05, 0.27, 0.1),
        emissive: Color::srgb(0.0, 1.0, 0.4),
        ..RobotStyle::default()
    }
}

pub fn scout_companion() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Ally,
        scale: 0.85,
        torso_width: 15.0, torso_height: 32.0, torso_depth: 12.0,
        leg_style: LegStyle::Digitigrade,
        has_cannons: true, cannon_size: 3.0,
        has_visor: true, visor_style: VisorStyle::Slit,
        primary: Color::srgb(0.15, 0.55, 0.15),
        secondary: Color::srgb(0.07, 0.27, 0.07),
        emissive: Color::srgb(0.0, 0.9, 0.2),
        ..RobotStyle::default()
    }
}

// ── Pet Presets ───────────────────────────────────────────────────────────────

pub fn spark_pup() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Pet,
        scale: 0.45,
        torso_width: 12.0, torso_height: 20.0, torso_depth: 10.0,
        head_size: 8.0, head_shape: HeadShape::Box,
        has_tail: true, tail_length: 18.0, tail_segments: 4,
        has_antennae: true, antenna_length: 14.0,
        has_visor: true, visor_style: VisorStyle::Round,
        primary: Color::srgb(0.85, 0.7, 0.1),
        secondary: Color::srgb(0.5, 0.4, 0.05),
        emissive: Color::srgb(1.0, 0.9, 0.0),
        ..RobotStyle::default()
    }
}

pub fn neon_cat() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Pet,
        scale: 0.4,
        torso_width: 11.0, torso_height: 18.0, torso_depth: 9.0,
        head_size: 7.0, head_shape: HeadShape::Sphere,
        leg_style: LegStyle::Digitigrade,
        has_tail: true, tail_length: 22.0, tail_segments: 5,
        has_horns: true, horn_length: 8.0,
        has_visor: true, visor_style: VisorStyle::Slit,
        primary: Color::srgb(0.35, 0.05, 0.5),
        secondary: Color::srgb(0.18, 0.02, 0.25),
        emissive: Color::srgb(0.8, 0.0, 1.0),
        ..RobotStyle::default()
    }
}

pub fn hover_orb() -> RobotStyle {
    RobotStyle {
        archetype: RobotArchetype::Pet,
        scale: 0.35,
        torso_width: 10.0, torso_height: 10.0, torso_depth: 10.0,
        head_size: 8.0, head_shape: HeadShape::Sphere,
        leg_style: LegStyle::Hoverpads,
        has_wings: true, wing_span: 20.0, wing_angle: 30.0,
        has_visor: true, visor_style: VisorStyle::Full,
        primary: Color::srgb(0.1, 0.7, 0.8),
        secondary: Color::srgb(0.05, 0.35, 0.4),
        emissive: Color::srgb(0.0, 1.0, 1.0),
        ..RobotStyle::default()
    }
}

// ── Lookup by Name ────────────────────────────────────────────────────────────
pub fn robot_by_name(name: &str) -> Option<RobotStyle> {
    match name {
        "ScoutPrime" => Some(scout_prime()),
        "BruteForge" => Some(brute_forge()),
        "JetWarden" => Some(jet_warden()),
        "TankTitan" => Some(tank_titan()),
        "InsectoidStalker" => Some(insectoid_stalker()),
        "HybridOmega" => Some(hybrid_omega()),
        "GuardianUnit" => Some(guardian_unit()),
        "MedicDrone" => Some(medic_drone_style()),
        "ScoutCompanion" => Some(scout_companion()),
        "SparkPup" => Some(spark_pup()),
        "NeonCat" => Some(neon_cat()),
        "HoverOrb" => Some(hover_orb()),
        _ => None,
    }
}
