use bevy::prelude::*;
use rand::Rng;

use crate::state::AppState;
use crate::components::world::*;
use crate::resources::GameSettings;

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), generate_city)
           .add_systems(OnExit(AppState::Playing), cleanup_world);
    }
}

fn cleanup_world(mut commands: Commands, q: Query<Entity, With<WorldGeometry>>) {
    for e in q.iter() { commands.entity(e).despawn_recursive(); }
}

// ── World generation entry point ──────────────────────────────────────────────
fn generate_city(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<GameSettings>,
) {
    let seed = settings.world_seed;

    // Lighting
    spawn_lighting(&mut commands);

    // Ground (1200×1200)
    spawn_ground(&mut commands, &mut meshes, &mut materials);

    // Zones
    spawn_downtown(&mut commands, &mut meshes, &mut materials, seed);
    spawn_industrial(&mut commands, &mut meshes, &mut materials, seed + 1);
    spawn_residential(&mut commands, &mut meshes, &mut materials, seed + 2);
    spawn_highways(&mut commands, &mut meshes, &mut materials);
    spawn_sky_platforms(&mut commands, &mut meshes, &mut materials, seed + 3);
    spawn_sky_bridges(&mut commands, &mut meshes, &mut materials, seed + 4);
    spawn_spaceports(&mut commands, &mut meshes, &mut materials);
    spawn_mountains(&mut commands, &mut meshes, &mut materials, seed + 5);
    spawn_neon_lights(&mut commands, seed + 6);
    spawn_street_lights(&mut commands, seed + 7);
    spawn_outer_districts(&mut commands, &mut meshes, &mut materials, seed + 8);
    spawn_river(&mut commands, &mut meshes, &mut materials);
}

// ── Seeded RNG helper ─────────────────────────────────────────────────────────
fn seeded(seed: u64, index: u64) -> f32 {
    let x = ((seed.wrapping_mul(127) + index.wrapping_mul(311)).wrapping_mul(43758)) as f64;
    let frac = (x * 0.0000001) - (x * 0.0000001).floor();
    frac as f32
}

// ── Materials ─────────────────────────────────────────────────────────────────
fn building_mat(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
    emissive: Option<Color>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        metallic: 0.3,
        perceptual_roughness: 0.7,
        emissive: emissive.map(|c| c.into()).unwrap_or(LinearRgba::BLACK),
        ..default()
    })
}

// ── Lighting ──────────────────────────────────────────────────────────────────
fn spawn_lighting(commands: &mut Commands) {
    // Ambient (hemisphere)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(
            Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)
        ),
        ..default()
    });

    // Neon accent
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::srgb(0.0, 0.8, 1.0),
            intensity: 500_000.0,
            range: 400.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 200.0, 0.0),
        ..default()
    });
}

// ── Ground ────────────────────────────────────────────────────────────────────
fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.10),
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: Mesh3d(meshes.add(Cuboid::new(1200.0, 0.5, 1200.0))),
            material: MeshMaterial3d(mat),
            transform: Transform::from_xyz(0.0, -0.25, 0.0),
            ..default()
        },
        WorldGeometry,
        WalkableSurface,
        Building { zone: WorldZone::Ground, height: 0.5 },
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(600.0, 0.25, 600.0),
    ));
}

// ── Downtown ──────────────────────────────────────────────────────────────────
fn spawn_downtown(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    let colors = [
        Color::srgb(0.12, 0.14, 0.22),
        Color::srgb(0.10, 0.12, 0.20),
        Color::srgb(0.15, 0.16, 0.25),
        Color::srgb(0.08, 0.10, 0.18),
    ];
    let emissives = [
        Some(Color::srgb(0.0, 0.4, 0.8)),
        Some(Color::srgb(0.8, 0.2, 0.0)),
        Some(Color::srgb(0.0, 0.8, 0.4)),
        None,
    ];

    let mut rng = rand::thread_rng();
    for i in 0..60u64 {
        let x = seeded(seed, i * 3) * 200.0 - 100.0;
        let z = seeded(seed, i * 3 + 1) * 200.0 - 100.0;
        let h = 30.0 + seeded(seed, i * 3 + 2) * 120.0;
        let w = 12.0 + rng.gen_range(0.0f32..18.0);
        let d = 12.0 + rng.gen_range(0.0f32..18.0);

        let ci = (i % 4) as usize;
        let mat = building_mat(materials, colors[ci], emissives[ci]);

        spawn_building(commands, meshes, mat, Vec3::new(x, h * 0.5, z), w, h, d, WorldZone::Downtown);
    }
}

// ── Industrial ────────────────────────────────────────────────────────────────
fn spawn_industrial(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    for i in 0..35u64 {
        let x = 120.0 + seeded(seed, i * 3) * 200.0;
        let z = seeded(seed, i * 3 + 1) * 400.0 - 200.0;
        let h = 15.0 + seeded(seed, i * 3 + 2) * 40.0;
        let w = 20.0 + seeded(seed, i * 4) * 30.0;
        let d = 20.0 + seeded(seed, i * 4 + 1) * 30.0;

        let mat = building_mat(materials, Color::srgb(0.18, 0.14, 0.10), Some(Color::srgb(1.0, 0.3, 0.0)));
        spawn_building(commands, meshes, mat, Vec3::new(x, h * 0.5, z), w, h, d, WorldZone::Industrial);

        // Chimney
        let cmat = building_mat(materials, Color::srgb(0.22, 0.18, 0.14), Some(Color::srgb(1.0, 0.5, 0.0)));
        spawn_building(commands, meshes, cmat, Vec3::new(x + 5.0, h + 10.0, z + 5.0), 3.0, 20.0, 3.0, WorldZone::Industrial);
    }
}

// ── Residential ───────────────────────────────────────────────────────────────
fn spawn_residential(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    for i in 0..50u64 {
        let x = -120.0 - seeded(seed, i * 3) * 200.0;
        let z = seeded(seed, i * 3 + 1) * 400.0 - 200.0;
        let h = 8.0 + seeded(seed, i * 3 + 2) * 25.0;
        let w = 10.0 + seeded(seed, i * 4) * 14.0;
        let d = 8.0 + seeded(seed, i * 4 + 1) * 14.0;

        let mat = building_mat(materials, Color::srgb(0.20, 0.18, 0.15), Some(Color::srgb(0.8, 0.7, 0.0)));
        spawn_building(commands, meshes, mat, Vec3::new(x, h * 0.5, z), w, h, d, WorldZone::Residential);
    }
}

// ── Highways ──────────────────────────────────────────────────────────────────
fn spawn_highways(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let road_mat = building_mat(materials, Color::srgb(0.15, 0.15, 0.18), None);

    // Main east-west highway at z=0
    commands.spawn((
        PbrBundle {
            mesh: Mesh3d(meshes.add(Cuboid::new(1200.0, 0.8, 18.0))),
            material: MeshMaterial3d(road_mat.clone()),
            transform: Transform::from_xyz(0.0, 8.0, 0.0),
            ..default()
        },
        WorldGeometry, WalkableSurface,
        Building { zone: WorldZone::Highway, height: 0.8 },
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(600.0, 0.4, 9.0),
    ));

    // North-south cross
    commands.spawn((
        PbrBundle {
            mesh: Mesh3d(meshes.add(Cuboid::new(18.0, 0.8, 1200.0))),
            material: MeshMaterial3d(road_mat.clone()),
            transform: Transform::from_xyz(0.0, 6.0, 0.0),
            ..default()
        },
        WorldGeometry, WalkableSurface,
        Building { zone: WorldZone::Highway, height: 0.8 },
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(9.0, 0.4, 600.0),
    ));

    // Pillars
    let pillar_mat = building_mat(materials, Color::srgb(0.2, 0.2, 0.25), None);
    for i in -6..=6i32 {
        let x = i as f32 * 100.0;
        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(meshes.add(Cuboid::new(2.5, 8.0, 2.5))),
                material: MeshMaterial3d(pillar_mat.clone()),
                transform: Transform::from_xyz(x, 4.0, 0.0),
                ..default()
            },
            WorldGeometry,
            bevy_rapier3d::prelude::RigidBody::Fixed,
            bevy_rapier3d::prelude::Collider::cuboid(1.25, 4.0, 1.25),
        ));
    }
}

// ── Sky Platforms ──────────────────────────────────────────────────────────────
fn spawn_sky_platforms(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.12, 0.20),
        emissive: LinearRgba::new(0.0, 0.3, 0.6, 1.0),
        metallic: 0.5,
        ..default()
    });

    for i in 0..12u64 {
        let x = seeded(seed, i * 4) * 600.0 - 300.0;
        let y = 40.0 + seeded(seed, i * 4 + 1) * 210.0;
        let z = seeded(seed, i * 4 + 2) * 600.0 - 300.0;
        let size = 25.0 + seeded(seed, i * 4 + 3) * 40.0;

        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(meshes.add(Cylinder::new(size, 3.0))),
                material: MeshMaterial3d(mat.clone()),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            WorldGeometry,
            SkyPlatform,
            WalkableSurface,
            Building { zone: WorldZone::SkyPlatform, height: 3.0 },
            bevy_rapier3d::prelude::RigidBody::Fixed,
            bevy_rapier3d::prelude::Collider::cylinder(1.5, size),
        ));
    }
}

// ── Sky Bridges ───────────────────────────────────────────────────────────────
fn spawn_sky_bridges(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    let mat = building_mat(materials, Color::srgb(0.08, 0.10, 0.18), Some(Color::srgb(0.0, 0.5, 1.0)));

    for i in 0..8u64 {
        let x = seeded(seed, i * 3) * 400.0 - 200.0;
        let y = 60.0 + seeded(seed, i * 3 + 1) * 100.0;
        let z = seeded(seed, i * 3 + 2) * 400.0 - 200.0;

        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(meshes.add(Cuboid::new(80.0, 1.5, 6.0))),
                material: MeshMaterial3d(mat.clone()),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            WorldGeometry,
            WalkableSurface,
            Building { zone: WorldZone::SkyPlatform, height: 1.5 },
            bevy_rapier3d::prelude::RigidBody::Fixed,
            bevy_rapier3d::prelude::Collider::cuboid(40.0, 0.75, 3.0),
        ));
    }
}

// ── Spaceports ────────────────────────────────────────────────────────────────
fn spawn_spaceports(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let positions = [
        Vec3::new(350.0, 0.5, 350.0),
        Vec3::new(-350.0, 0.5, 350.0),
        Vec3::new(350.0, 0.5, -350.0),
        Vec3::new(-350.0, 0.5, -350.0),
    ];
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.15, 0.20),
        metallic: 0.7,
        emissive: LinearRgba::new(0.0, 0.4, 0.8, 1.0),
        ..default()
    });

    for pos in positions {
        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(meshes.add(Cylinder::new(50.0, 2.0))),
                material: MeshMaterial3d(mat.clone()),
                transform: Transform::from_translation(pos),
                ..default()
            },
            WorldGeometry,
            WalkableSurface,
            Building { zone: WorldZone::Spaceport, height: 2.0 },
            bevy_rapier3d::prelude::RigidBody::Fixed,
            bevy_rapier3d::prelude::Collider::cylinder(1.0, 50.0),
        ));
    }
}

// ── Mountains ─────────────────────────────────────────────────────────────────
fn spawn_mountains(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.28, 0.25),
        metallic: 0.1,
        perceptual_roughness: 0.9,
        ..default()
    });
    let snow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.92, 1.0),
        metallic: 0.0,
        perceptual_roughness: 0.8,
        ..default()
    });

    // 4 mountain ranges at world corners
    let corners = [
        Vec3::new(450.0, 0.0, 450.0),
        Vec3::new(-450.0, 0.0, 450.0),
        Vec3::new(450.0, 0.0, -450.0),
        Vec3::new(-450.0, 0.0, -450.0),
    ];

    for (ci, &corner) in corners.iter().enumerate() {
        for i in 0..15u64 {
            let idx = ci as u64 * 15 + i;
            let ox = seeded(seed, idx * 3) * 150.0 - 75.0;
            let oz = seeded(seed, idx * 3 + 1) * 150.0 - 75.0;
            let h = 40.0 + seeded(seed, idx * 3 + 2) * 120.0;
            let r = 20.0 + seeded(seed, idx * 4) * 30.0;
            let pos = corner + Vec3::new(ox, h * 0.5, oz);

            commands.spawn((
                PbrBundle {
                    mesh: Mesh3d(meshes.add(Cone { radius: r, height: h })),
                    material: MeshMaterial3d(rock_mat.clone()),
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                WorldGeometry,
            ));
            // Snow cap
            if h > 80.0 {
                let snow_pos = corner + Vec3::new(ox, h * 0.92 + 5.0, oz);
                commands.spawn((
                    PbrBundle {
                        mesh: Mesh3d(meshes.add(Cone { radius: r * 0.3, height: h * 0.15 })),
                        material: MeshMaterial3d(snow_mat.clone()),
                        transform: Transform::from_translation(snow_pos),
                        ..default()
                    },
                    WorldGeometry,
                ));
            }
        }
    }
}

// ── Neon Lights ───────────────────────────────────────────────────────────────
fn spawn_neon_lights(commands: &mut Commands, seed: u64) {
    let neon_colors = [
        Color::srgb(0.0, 1.0, 1.0),
        Color::srgb(1.0, 0.0, 0.8),
        Color::srgb(0.0, 1.0, 0.3),
        Color::srgb(1.0, 0.6, 0.0),
        Color::srgb(0.5, 0.0, 1.0),
    ];

    for i in 0..70u64 {
        let x = seeded(seed, i * 3) * 600.0 - 300.0;
        let y = 5.0 + seeded(seed, i * 3 + 1) * 60.0;
        let z = seeded(seed, i * 3 + 2) * 600.0 - 300.0;
        let ci = (i % 5) as usize;

        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    color: neon_colors[ci],
                    intensity: 20_000.0,
                    range: 25.0,
                    shadows_enabled: false,
                    ..default()
                },
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            WorldGeometry,
            NeonLight,
        ));
    }
}

// ── Street Lights ─────────────────────────────────────────────────────────────
fn spawn_street_lights(commands: &mut Commands, seed: u64) {
    for i in 0..80u64 {
        let x = seeded(seed, i * 2) * 400.0 - 200.0;
        let z = seeded(seed, i * 2 + 1) * 400.0 - 200.0;

        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    color: Color::srgb(0.9, 0.85, 0.7),
                    intensity: 8_000.0,
                    range: 18.0,
                    shadows_enabled: false,
                    ..default()
                },
                transform: Transform::from_xyz(x, 7.0, z),
                ..default()
            },
            WorldGeometry,
        ));
    }
}

// ── Outer Districts ───────────────────────────────────────────────────────────
fn spawn_outer_districts(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    let offsets = [
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(-300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 300.0),
        Vec3::new(0.0, 0.0, -300.0),
        Vec3::new(250.0, 0.0, -250.0),
    ];

    for (di, &offset) in offsets.iter().enumerate() {
        let mat = building_mat(
            materials,
            Color::srgb(0.12 + di as f32 * 0.02, 0.14, 0.20),
            Some(Color::srgb(di as f32 * 0.2, 0.4, 0.8)),
        );
        for i in 0..20u64 {
            let idx = di as u64 * 20 + i;
            let ox = seeded(seed, idx * 3) * 120.0 - 60.0;
            let oz = seeded(seed, idx * 3 + 1) * 120.0 - 60.0;
            let h = 10.0 + seeded(seed, idx * 3 + 2) * 50.0;
            let w = 8.0 + seeded(seed, idx * 4) * 15.0;
            let d = 8.0 + seeded(seed, idx * 4 + 1) * 15.0;

            spawn_building(commands, meshes, mat.clone(),
                offset + Vec3::new(ox, h * 0.5, oz), w, h, d, WorldZone::OuterDistrict);
        }
    }
}

// ── River ─────────────────────────────────────────────────────────────────────
fn spawn_river(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.2, 0.4, 0.8),
        metallic: 0.9,
        perceptual_roughness: 0.1,
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::new(0.0, 0.1, 0.3, 1.0),
        ..default()
    });

    // Sinusoidal river approximated with segments
    for i in -15..=15i32 {
        let z = i as f32 * 40.0;
        let x = (z * 0.05).sin() * 60.0;
        commands.spawn((
            PbrBundle {
                mesh: Mesh3d(meshes.add(Cuboid::new(25.0, 0.1, 42.0))),
                material: MeshMaterial3d(water_mat.clone()),
                transform: Transform::from_xyz(x, -0.1, z),
                ..default()
            },
            WorldGeometry,
        ));
    }
}

// ── Building helper ───────────────────────────────────────────────────────────
fn spawn_building(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
    position: Vec3,
    width: f32,
    height: f32,
    depth: f32,
    zone: WorldZone,
) {
    commands.spawn((
        PbrBundle {
            mesh: Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
            material: MeshMaterial3d(mat),
            transform: Transform::from_translation(position),
            ..default()
        },
        WorldGeometry,
        WalkableSurface,
        Building { zone, height },
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(width * 0.5, height * 0.5, depth * 0.5),
    ));
}
