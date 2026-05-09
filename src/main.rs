use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier3d::prelude::*;

mod state;
mod events;
mod damage;
mod resources;
mod components;
mod robots;
mod chapters;
mod perks;
mod lsystem;
mod plugins;

use state::AppState;
use events::EventsPlugin;
use plugins::{
    InputPlugin,
    PlayerPlugin,
    WeaponPlugin,
    EnemyPlugin,
    WorldPlugin,
    ChestPlugin,
    CompanionPlugin,
    ArmorPlugin,
    CraftingPlugin,
    UiPlugin,
    SavePlugin,
    ChapterPlugin,
    DiscoverablePlugin,
    RadioPlugin,
    VehiclePlugin,
    ChassisEditorPlugin,
};
use resources::{WaveInfo, GameSettings, PlayerScore, CameraShake};

fn main() {
    App::new()
        // Default plugins with window setup
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Heavy Water".to_string(),
                        resolution: WindowResolution::new(1280.0, 720.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // Physics
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // State
        .init_state::<AppState>()
        // Global resources
        .init_resource::<WaveInfo>()
        .init_resource::<GameSettings>()
        .init_resource::<PlayerScore>()
        .init_resource::<CameraShake>()
        // Event infrastructure
        .add_plugins(EventsPlugin)
        // Game plugins
        .add_plugins((
            InputPlugin,
            UiPlugin,
            WorldPlugin,
            PlayerPlugin,
            WeaponPlugin,
            EnemyPlugin,
            ChestPlugin,
            CompanionPlugin,
            ArmorPlugin,
            CraftingPlugin,
            SavePlugin,
        ))
        .add_plugins((
            ChapterPlugin,
            DiscoverablePlugin,
            RadioPlugin,
            VehiclePlugin,
            ChassisEditorPlugin,
        ))
        .run();
}
