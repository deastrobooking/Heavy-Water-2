use bevy::prelude::*;

use crate::state::AppState;
use crate::events::*;
use crate::components::player::{Player, PlayerStats, PlayerStateMachine, JetpackState};
use crate::components::weapon::WeaponInventory;
use crate::components::armor::ArmorSet;
use crate::damage::Health;
use crate::resources::{WaveInfo, UiMessage};

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UiMessage>()
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnEnter(AppState::Playing), (setup_hud, despawn_menu))
            .add_systems(OnEnter(AppState::GameOver), setup_game_over)
            .add_systems(
                Update,
                (
                    hud_update_system,
                    message_timer_system,
                    ui_message_listener,
                    game_over_input,
                )
                    .run_if(in_state(AppState::Playing).or_else(in_state(AppState::GameOver))),
            )
            .add_systems(Update, menu_start_button.run_if(in_state(AppState::MainMenu)));
    }
}

// ── UI Node Tags ──────────────────────────────────────────────────────────────
#[derive(Component)] struct MainMenuRoot;
#[derive(Component)] struct HudRoot;
#[derive(Component)] struct GameOverRoot;
#[derive(Component)] struct StartButton;
#[derive(Component)] struct GameOverText;
#[derive(Component)] struct MessageText;

// Bar fill markers (Default required for spawn_bar_row generic)
#[derive(Component, Default)] struct HealthBar;
#[derive(Component, Default)] struct ArmorBar;
#[derive(Component, Default)] struct StaminaBar;
#[derive(Component, Default)] struct JetpackBar;

// Text markers
#[derive(Component)] struct CreditsText;
#[derive(Component)] struct LevelText;
#[derive(Component)] struct ElementBadge;
#[derive(Component)] struct EnemyCountText;
#[derive(Component)] struct WaveText;
#[derive(Component)] struct WeaponNameText;
#[derive(Component)] struct AmmoText;

fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MainMenuRoot>>) {
    for e in q.iter() { commands.entity(e).despawn_recursive(); }
}

// ── Main Menu ─────────────────────────────────────────────────────────────────
fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(0.01, 0.01, 0.05, 1.0).into(),
            ..default()
        },
        MainMenuRoot,
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "DETROIT 3026",
            TextStyle {
                font_size: 72.0,
                color: Color::srgb(0.0, 0.8, 1.0),
                ..default()
            },
        ));

        parent.spawn(TextBundle::from_section(
            "Open World Action RPG",
            TextStyle {
                font_size: 24.0,
                color: Color::srgb(0.6, 0.6, 0.8),
                ..default()
            },
        ));

        // Spacer
        parent.spawn(NodeBundle { style: Style { height: Val::Px(40.0), ..default() }, ..default() });

        // Start button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                background_color: Color::srgb(0.0, 0.4, 0.8).into(),
                ..default()
            },
            StartButton,
        )).with_children(|btn| {
            btn.spawn(TextBundle::from_section(
                "START MISSION",
                TextStyle { font_size: 28.0, color: Color::WHITE, ..default() },
            ));
        });

        // Controls hint
        parent.spawn(NodeBundle { style: Style { height: Val::Px(30.0), ..default() }, ..default() });
        parent.spawn(TextBundle::from_section(
            "WASD Move  •  Mouse Look  •  LMB Fire  •  V/B Melee  •  T Beam Sabre  •  F Parry  •  Q Dodge  •  Space Jump/Jetpack",
            TextStyle { font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.7), ..default() },
        ));
    });
}

fn menu_start_button(
    mut interaction_q: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in interaction_q.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Playing);
        }
    }
}

// ── HUD Setup ─────────────────────────────────────────────────────────────────
fn setup_hud(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        HudRoot,
    )).with_children(|root| {
        // ── Top-Left: health/armor/stamina/jetpack/credits/level ──────────────
        root.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                width: Val::Px(220.0),
                ..default()
            },
            ..default()
        }).with_children(|panel| {
            spawn_bar_row(panel, "HP", HealthBar, Color::srgb(0.2, 0.8, 0.2));
            spawn_bar_row(panel, "AR", ArmorBar, Color::srgb(0.2, 0.5, 1.0));
            spawn_bar_row(panel, "ST", StaminaBar, Color::srgb(0.9, 0.7, 0.0));
            spawn_bar_row(panel, "JP", JetpackBar, Color::srgb(0.0, 0.9, 0.9));

            // Credits
            panel.spawn((
                TextBundle::from_section("¢ 0", TextStyle { font_size: 16.0, color: Color::srgb(0.9, 0.75, 0.1), ..default() }),
                CreditsText,
            ));

            // Level
            panel.spawn((
                TextBundle::from_section("LVL 1", TextStyle { font_size: 16.0, color: Color::srgb(0.7, 0.4, 1.0), ..default() }),
                LevelText,
            ));

            // Element
            panel.spawn((
                TextBundle::from_section("Element: None", TextStyle { font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.9), ..default() }),
                ElementBadge,
            ));
        });

        // ── Top-Right: enemy count / wave ─────────────────────────────────────
        root.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(6.0),
                ..default()
            },
            ..default()
        }).with_children(|panel| {
            panel.spawn((
                TextBundle::from_section("Wave 1", TextStyle { font_size: 20.0, color: Color::srgb(1.0, 0.4, 0.2), ..default() }),
                WaveText,
            ));
            panel.spawn((
                TextBundle::from_section("Enemies: 0", TextStyle { font_size: 16.0, color: Color::srgb(1.0, 0.7, 0.3), ..default() }),
                EnemyCountText,
            ));
        });

        // ── Bottom-Left: weapon name / ammo ───────────────────────────────────
        root.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                bottom: Val::Px(80.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ..default()
        }).with_children(|panel| {
            panel.spawn((
                TextBundle::from_section("Plasma Pistol", TextStyle { font_size: 22.0, color: Color::srgb(0.0, 0.8, 1.0), ..default() }),
                WeaponNameText,
            ));
            panel.spawn((
                TextBundle::from_section("50 / 50", TextStyle { font_size: 18.0, color: Color::srgb(0.8, 0.8, 0.8), ..default() }),
                AmmoText,
            ));
        });

        // ── Center: crosshair ─────────────────────────────────────────────────
        root.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect::all(Val::Px(-8.0)),
                width: Val::Px(16.0),
                height: Val::Px(16.0),
                ..default()
            },
            ..default()
        }).with_children(|ch| {
            // Horizontal line
            ch.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(7.0),
                    width: Val::Px(16.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });
            // Vertical line
            ch.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(7.0),
                    top: Val::Px(0.0),
                    width: Val::Px(2.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });
        });

        // ── Upper-Center: message popup ────────────────────────────────────────
        root.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(80.0),
                ..default()
            },
            ..default()
        }).with_children(|msg| {
            msg.spawn((
                TextBundle::from_section("", TextStyle { font_size: 22.0, color: Color::srgb(1.0, 0.9, 0.3), ..default() }),
                MessageText,
            ));
        });

        // ── Weapon selector bar (bottom) ──────────────────────────────────────
        root.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Percent(50.0),
                column_gap: Val::Px(4.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).with_children(|bar| {
            let weapon_names = ["1:Pistol","2:Rifle","3:Shotgun","4:Rocket","5:Laser","6:Grenade"];
            for (i, name) in weapon_names.iter().enumerate() {
                bar.spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(4.0)),
                        width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.1, 0.1, 0.2, 0.7).into(),
                    ..default()
                }).with_children(|slot| {
                    slot.spawn(TextBundle::from_section(
                        *name,
                        TextStyle { font_size: 11.0, color: Color::srgb(0.7, 0.8, 1.0), ..default() },
                    ));
                });
            }
        });
    });
}

fn spawn_bar_row<L: Component + Default>(
    parent: &mut ChildBuilder,
    label: &str,
    _bar_marker: L,
    color: Color,
) {
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },
        ..default()
    }).with_children(|row| {
        row.spawn(TextBundle::from_section(label, TextStyle { font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.8), ..default() }));
        // Bar background
        row.spawn(NodeBundle {
            style: Style { width: Val::Px(150.0), height: Val::Px(12.0), ..default() },
            background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
            ..default()
        }).with_children(|bg| {
            bg.spawn((
                NodeBundle {
                    style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                    background_color: color.into(),
                    ..default()
                },
                L::default(),
            ));
        });
    });
}

// ── HUD Update ────────────────────────────────────────────────────────────────
fn hud_update_system(
    player_q: Query<(&Health, &PlayerStats, &JetpackState, &WeaponInventory, &ArmorSet, &PlayerStateMachine), With<Player>>,
    wave: Res<WaveInfo>,
    mut health_bar_q: Query<&mut Style, With<HealthBar>>,
    mut armor_bar_q: Query<&mut Style, (With<ArmorBar>, Without<HealthBar>)>,
    mut stamina_bar_q: Query<&mut Style, (With<StaminaBar>, Without<HealthBar>, Without<ArmorBar>)>,
    mut jp_bar_q: Query<&mut Style, (With<JetpackBar>, Without<HealthBar>, Without<ArmorBar>, Without<StaminaBar>)>,
    mut credits_q: Query<&mut Text, (With<CreditsText>, Without<WaveText>, Without<EnemyCountText>, Without<WeaponNameText>, Without<AmmoText>, Without<LevelText>, Without<ElementBadge>)>,
    mut level_q: Query<&mut Text, (With<LevelText>, Without<CreditsText>, Without<WaveText>, Without<EnemyCountText>, Without<WeaponNameText>, Without<AmmoText>, Without<ElementBadge>)>,
    mut element_q: Query<&mut Text, (With<ElementBadge>, Without<CreditsText>, Without<WaveText>, Without<EnemyCountText>, Without<WeaponNameText>, Without<AmmoText>, Without<LevelText>)>,
    mut wave_q: Query<&mut Text, (With<WaveText>, Without<EnemyCountText>, Without<CreditsText>, Without<WeaponNameText>, Without<AmmoText>, Without<LevelText>, Without<ElementBadge>)>,
    mut enemy_q: Query<&mut Text, (With<EnemyCountText>, Without<WaveText>, Without<CreditsText>, Without<WeaponNameText>, Without<AmmoText>, Without<LevelText>, Without<ElementBadge>)>,
    mut weapon_name_q: Query<&mut Text, (With<WeaponNameText>, Without<AmmoText>, Without<WaveText>, Without<EnemyCountText>, Without<CreditsText>, Without<LevelText>, Without<ElementBadge>)>,
    mut ammo_q: Query<&mut Text, (With<AmmoText>, Without<WeaponNameText>, Without<WaveText>, Without<EnemyCountText>, Without<CreditsText>, Without<LevelText>, Without<ElementBadge>)>,
) {
    let Ok((health, stats, jetpack, weapons, armor, _sm)) = player_q.get_single() else { return };

    // Health bar width
    if let Ok(mut style) = health_bar_q.get_single_mut() {
        style.width = Val::Percent((health.current / health.max * 100.0).clamp(0.0, 100.0));
    }
    // Armor bar
    if let Ok(mut style) = armor_bar_q.get_single_mut() {
        style.width = Val::Percent((stats.armor / stats.max_armor * 100.0).clamp(0.0, 100.0));
    }
    // Stamina bar
    if let Ok(mut style) = stamina_bar_q.get_single_mut() {
        style.width = Val::Percent((stats.stamina / stats.max_stamina * 100.0).clamp(0.0, 100.0));
    }
    // Jetpack bar
    if let Ok(mut style) = jp_bar_q.get_single_mut() {
        style.width = Val::Percent((jetpack.fuel / jetpack.max_fuel * 100.0).clamp(0.0, 100.0));
    }
    // Credits
    if let Ok(mut text) = credits_q.get_single_mut() {
        text.sections[0].value = format!("¢ {}", stats.credits);
    }
    // Level
    if let Ok(mut text) = level_q.get_single_mut() {
        let needed = stats.xp_for_next_level();
        text.sections[0].value = format!("LVL {}  XP {}/{}", stats.level, stats.experience, needed);
    }
    // Element
    if let Ok(mut text) = element_q.get_single_mut() {
        text.sections[0].value = format!("Element: {}", armor.active_element.display_name());
    }
    // Wave
    if let Ok(mut text) = wave_q.get_single_mut() {
        text.sections[0].value = format!("Wave {}", wave.wave_number);
    }
    // Enemy count
    if let Ok(mut text) = enemy_q.get_single_mut() {
        text.sections[0].value = format!("Enemies: {}", wave.enemy_count);
    }
    // Weapon
    let weapon = weapons.active();
    if let Ok(mut text) = weapon_name_q.get_single_mut() {
        text.sections[0].value = weapon.weapon_type.display_name().to_string();
    }
    if let Ok(mut text) = ammo_q.get_single_mut() {
        text.sections[0].value = format!("{} / {}", weapon.ammo, weapon.max_ammo);
    }
}

// ── Message System ────────────────────────────────────────────────────────────
fn message_timer_system(
    time: Res<Time>,
    mut msg: ResMut<UiMessage>,
    mut text_q: Query<&mut Text, With<MessageText>>,
) {
    if msg.timer > 0.0 {
        msg.timer -= time.delta_seconds();
        if msg.timer <= 0.0 {
            msg.text.clear();
            if let Ok(mut t) = text_q.get_single_mut() { t.sections[0].value.clear(); }
        }
    }
}

fn ui_message_listener(
    mut ev: EventReader<UiMessageEvent>,
    mut msg: ResMut<UiMessage>,
    mut text_q: Query<&mut Text, With<MessageText>>,
) {
    for e in ev.read() {
        msg.text = e.text.clone();
        msg.timer = e.duration;
        if let Ok(mut t) = text_q.get_single_mut() {
            t.sections[0].value = e.text.clone();
        }
    }
}

// ── Game Over ─────────────────────────────────────────────────────────────────
fn setup_game_over(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
            ..default()
        },
        GameOverRoot,
    )).with_children(|p| {
        p.spawn((
            TextBundle::from_section(
                "SYSTEM FAILURE",
                TextStyle { font_size: 64.0, color: Color::srgb(1.0, 0.2, 0.1), ..default() },
            ),
            GameOverText,
        ));
        p.spawn(TextBundle::from_section(
            "Press R to restart",
            TextStyle { font_size: 24.0, color: Color::srgb(0.7, 0.7, 0.8), ..default() },
        ));
    });
}

fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    go_root: Query<Entity, With<GameOverRoot>>,
    hud_root: Query<Entity, With<HudRoot>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for e in go_root.iter() { commands.entity(e).despawn_recursive(); }
        for e in hud_root.iter() { commands.entity(e).despawn_recursive(); }
        next_state.set(AppState::MainMenu);
    }
}
