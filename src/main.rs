use bevy::audio::{AudioSink, PlaybackMode, PlaybackSettings, Volume};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use rand::Rng;

/* =======================
   COMPONENTS
======================= */
#[derive(Component)]
struct MenuBackground;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Present;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct HeatText;

#[derive(Component)]
#[allow(dead_code)]
struct MessageText {
    alpha: f32,
}
#[derive(Resource)]
struct HitFreeze {
    timer: Timer,
    active: bool,
}
#[derive(Component)]
struct MenuUI;
#[derive(Resource)]
struct GameOverFade {
    alpha: f32,
}

#[derive(Component)]
struct Snowflake;

#[derive(Resource, PartialEq)]
enum GameState {
    Menu,
    Playing,
    Crashed,
}
#[derive(Resource)]
struct ScreenShake {
    intensity: f32,
}
#[derive(Resource)]
struct CrashSoundPlayed(bool);

#[derive(Component)]
struct Fan;
#[derive(Component)]
struct FanCollected;

#[derive(Resource)]
struct FanSpawnTimer(Timer);
#[derive(Resource)]
struct SnowSpawnTimer(Timer);
#[derive(Component)]
struct SnowSpeed(f32);

#[derive(Component)]
struct CorruptedBit;

#[derive(Resource)]
#[allow(dead_code)]
struct CorruptedSpawnTimer(Timer);

#[derive(Resource)]
struct CorruptedBitSpawnTimer(Timer);

#[derive(Component)]
struct BackgroundMusic;
#[derive(Resource)]
struct Difficulty {
    level: f32,
    time_alive: f32,
}
#[derive(Resource)]
struct TimeScale {
    value: f32,
}

#[derive(Component)]
struct GameOverOverlay;

#[derive(Resource)]
struct GameData {
    score: u32,
    heat: f32,
    speed_multiplier: f32,
    overloading: bool,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            heat: 0.0,
            speed_multiplier: 1.0,
            overloading: false,
        }
    }
}
#[derive(Resource)]
struct SpawnTimer(Timer);

/* =======================
   MAIN
======================= */

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameData::default())
        .insert_resource(GameState::Menu)
        .insert_resource(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(ScreenShake { intensity: 0.0 })
        .insert_resource(FanSpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        .insert_resource(CrashSoundPlayed(false))

        .insert_resource(CorruptedSpawnTimer(Timer::from_seconds(
            2.5,
            TimerMode::Repeating,
        )))
        .insert_resource(CorruptedBitSpawnTimer(Timer::from_seconds(
            2.5,
            TimerMode::Repeating,
        )))
        .insert_resource(Difficulty {
            level: 1.0,
            time_alive: 0.0,
        })
        .insert_resource(HitFreeze {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
            active: false,
        })
        .insert_resource(TimeScale { value: 1.0 })
        .insert_resource(GameOverFade { alpha: 0.0 })
        .insert_resource(SnowSpawnTimer(Timer::from_seconds(
            0.05,
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup, setup_ui, setup_menu, play_music))
        .add_systems(Update, set_background_music_volume)
        .add_systems(Update, move_snow)
        .add_systems(
            Update,
            (
                menu_input,
                update_difficulty,
                player_movement,
                overload_system,
                spawn_presents,
                move_presents,
                collect_presents,
            ),
        )
        .add_systems(
            Update,
            (
                spawn_fans,
                move_fans,
                collect_fans,
                spawn_corrupted_bits,
                move_corrupted_bits,
                hit_corrupted_bits,
            ),
        )
        .add_systems(
            Update,
            (
                corrupted_bit_hit_sound,
                restore_background_music,
                camera_shake,
            ),
        )
        .add_systems(Update, near_crash_slow_motion)
        .add_systems(Update, spawn_snowflakes)
        .add_systems(
            Update,
            (
                crash_check,
                spawn_game_over_overlay,
                fade_game_over,
                update_ui,
                restart_game,
            ),
        )
        .add_systems(Update, hit_freeze_system)
        .run();
}

/* =======================
   SETUP
======================= */
fn set_background_music_volume(
    state: Res<GameState>,
    bg_music: Query<&AudioSink, With<BackgroundMusic>>,
) {
    let Ok(sink) = bg_music.get_single() else {
        return;
    };

    match *state {
        GameState::Menu => {
            sink.set_volume(0.35); // calm menu
        }
        GameState::Playing => {
            sink.set_volume(0.55); // main gameplay
        }
        GameState::Crashed => {
            sink.set_volume(0.0); // silence on crash
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // 🎅 Santa sprite
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/santa.png"),
            transform: Transform {
                translation: Vec3::new(0.0, -250.0, 0.0),
                scale: Vec3::splat(1.0),
                ..default()
            },
            visibility: Visibility::Hidden, // 👈 correct
            ..default()
        },
        Player,
    ));
}

fn camera_shake(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    data: Res<GameData>,
    state: Res<GameState>,
    mut shake: ResMut<ScreenShake>,
) {
    if *state != GameState::Playing {
        return;
    }

    let Ok(mut cam) = camera.get_single_mut() else {
        return;
    };

    // Increase shake when overloading
    if data.overloading {
        shake.intensity = (shake.intensity + 0.8).clamp(0.0, 6.0);
    } else {
        shake.intensity *= 0.9;
    }

    // ✅ SAFETY CHECK (THIS FIXES THE CRASH)
    if shake.intensity < 0.01 {
        cam.translation.x = 0.0;
        cam.translation.y = 0.0;
        shake.intensity = 0.0;
        return;
    }

    let mut rng = rand::thread_rng();
    cam.translation.x = rng.gen_range(-shake.intensity..shake.intensity);
    cam.translation.y = rng.gen_range(-shake.intensity..shake.intensity);
}
fn spawn_fans(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<GameState>,
    time: Res<Time>,
    mut timer: ResMut<FanSpawnTimer>,
    difficulty: Res<Difficulty>, // ✅ REQUIRED
) {
    if *state != GameState::Playing {
        return;
    }

    timer.0.tick(time.delta());

    timer.0.set_duration(std::time::Duration::from_secs_f32(
        (5.0_f32 * difficulty.level).clamp(3.0_f32, 8.0_f32),
    ));

    if timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-300.0_f32..300.0_f32);

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites/cooling.png"),
                sprite: Sprite {
                    color: Color::WHITE,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x, 300.0, 0.0),
                    scale: Vec3::splat(0.6),
                    ..default()
                },
                ..default()
            },
            Fan,
        ));
    }
}

fn move_fans(
    mut commands: Commands,
    state: Res<GameState>,
    difficulty: Res<Difficulty>,
    time_scale: Res<TimeScale>,
    mut query: Query<(Entity, &mut Transform), With<Fan>>,
) {
    if *state != GameState::Playing {
        return;
    }

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.y -= 2.0 * difficulty.level * time_scale.value;

        if transform.translation.y < -360.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn collect_fans(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    state: Res<GameState>,
    player: Query<&Transform, With<Player>>,
    fans: Query<(Entity, &Transform), (With<Fan>, Without<FanCollected>)>,
    asset_server: Res<AssetServer>,
) {
    if *state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (entity, transform) in fans.iter() {
        if player_pos.distance(transform.translation) < 40.0 {
            // ❄ Reduce heat
            data.heat = (data.heat - 25.0).clamp(0.0, 100.0);

            // 🔊 Play fan sound ONCE
            commands.spawn(AudioBundle {
                source: asset_server.load("audio/Wind2.wav"),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::new(0.4), // clean & soft
                    ..default()
                },
            });

            // 🔒 Mark as collected immediately
            commands.entity(entity).insert(FanCollected);

            // 🧹 Despawn safely
            commands.entity(entity).despawn_recursive();

            break; // ✅ VERY IMPORTANT
        }
    }
}


fn spawn_corrupted_bits(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<GameState>,
    time: Res<Time>,
    mut timer: ResMut<CorruptedBitSpawnTimer>,
    difficulty: Res<Difficulty>, // ✅ REQUIRED
) {
    if *state != GameState::Playing {
        return;
    }

    timer.0.tick(time.delta());

    timer.0.set_duration(std::time::Duration::from_secs_f32(
        (2.5_f32 / difficulty.level).clamp(0.6_f32, 2.5_f32),
    ));

    if timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-300.0_f32..300.0_f32);

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites/corrupted.png"),
                sprite: Sprite {
                    color: Color::WHITE,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x, 300.0, 0.0),
                    scale: Vec3::splat(0.8),
                    ..default()
                },
                ..default()
            },
            CorruptedBit,
        ));
    }
}

fn move_corrupted_bits(
    mut commands: Commands,
    state: Res<GameState>,
    data: Res<GameData>,
    difficulty: Res<Difficulty>,
    time_scale: Res<TimeScale>,
    mut query: Query<(Entity, &mut Transform), With<CorruptedBit>>,
) {
    if *state != GameState::Playing {
        return;
    }

    if *state == GameState::Crashed {
        return;
    }

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.y -=
            4.5 * difficulty.level * data.speed_multiplier * time_scale.value;

        if transform.translation.y < -350.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn hit_corrupted_bits(
    mut commands: Commands,
    state: Res<GameState>,
    mut freeze: ResMut<HitFreeze>,
    player: Query<&Transform, With<Player>>,
    bits: Query<(Entity, &Transform), With<CorruptedBit>>,
    asset_server: Res<AssetServer>,
) {
    if *state != GameState::Playing || freeze.active {
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, bit_transform) in bits.iter() {
        if player_pos.distance(bit_transform.translation) < 35.0 {
            // ❄️ HIT FREEZE
            freeze.active = true;
            freeze.timer.reset();

            // 🔊 SYSTEM FAILURE SOUND
            commands.spawn(AudioBundle {
                source: asset_server.load("audio/GameOver2.wav"),
                settings: PlaybackSettings {
                    volume: Volume::new(0.9), // 🎬 cinematic punch
                    ..default()
                },
            });

            // 🧹 REMOVE BIT
            commands.entity(entity).despawn();
            break;
        }
    }
}

fn hit_freeze_system(time: Res<Time>, mut freeze: ResMut<HitFreeze>, mut state: ResMut<GameState>) {
    if *state != GameState::Playing {
        return;
    }

    if !freeze.active {
        return;
    }

    freeze.timer.tick(time.delta());

    if freeze.timer.finished() {
        freeze.active = false;
        *state = GameState::Crashed;
        println!("💥 SYSTEM FAILURE");
    }
}

fn corrupted_bit_hit_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bg_music: Query<&AudioSink, With<BackgroundMusic>>,
    mut played: ResMut<CrashSoundPlayed>,
    state: Res<GameState>,
) {
    // Only play once, only on crash
    if *state != GameState::Crashed || played.0 {
        return;
    }

    // 🔇 Mute background music
    if let Ok(sink) = bg_music.get_single() {
        sink.set_volume(0.0);
    }

    // 🔊 Play crash sound ONCE
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/GameOver2.wav"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.9), // clean, not harsh
            ..default()
        },
    });

    played.0 = true; // 🔒 LOCK IT
}

fn restore_background_music(bg_music: Query<&AudioSink, With<BackgroundMusic>>) {
    if let Ok(sink) = bg_music.get_single() {
        sink.set_volume(0.3); // 🔉 restore background
    }
}

fn update_difficulty(
    time: Res<Time>,
    data: Res<GameData>,
    mut difficulty: ResMut<Difficulty>,
    state: Res<GameState>,
) {
    if *state != GameState::Playing {
        return;
    }

    // ⏱ Track survival time
    difficulty.time_alive += time.delta_seconds();

    // 🐢 VERY SLOW BASE RAMP (Subway Surfers style)
    let base_growth = if difficulty.time_alive < 60.0 {
        0.002 // first 1 minute = chill
    } else if difficulty.time_alive < 120.0 {
        0.004 // minute 2–3 = tension
    } else {
        0.006 // late game
    };

    difficulty.level += base_growth * time.delta_seconds();

    // 🔥 Heat only matters AFTER 90 seconds
    if difficulty.time_alive > 90.0 {
        let heat_factor = (data.heat / 100.0).powf(1.8);
        difficulty.level += heat_factor * 0.003;
    }

    // 🛑 Hard cap (never unfair)
    difficulty.level = difficulty.level.clamp(1.0, 3.2);
}

fn near_crash_slow_motion(
    mut time_scale: ResMut<TimeScale>,
    data: Res<GameData>,
    state: Res<GameState>,
) {
    if *state != GameState::Playing {
        time_scale.value = 0.0;
        return;
    }

    // 🔥 Near crash zone
    if data.heat >= 85.0 {
        let t = (data.heat - 85.0) / 15.0; // 0 → 1
        time_scale.value = (1.0 - t * 0.5).clamp(0.5, 1.0);
    } else {
        // Smooth recovery
        time_scale.value += (1.0 - time_scale.value) * 0.05;
    }
}
fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    mut commands: Commands,
    menu_ui: Query<Entity, With<MenuUI>>,
    menu_bg: Query<Entity, With<MenuBackground>>,
    mut player: Query<&mut Visibility, With<Player>>,
) {
    if *state != GameState::Menu {
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        *state = GameState::Playing;

        // 🧹 Remove menu UI
        for e in menu_ui.iter() {
            commands.entity(e).despawn_recursive();
        }

        // 🧹 Remove menu background
        for e in menu_bg.iter() {
            commands.entity(e).despawn_recursive();
        }

        // 👀 SHOW SANTA
        if let Ok(mut vis) = player.get_single_mut() {
            *vis = Visibility::Visible;
        }

        println!("▶ GAME STARTED");
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("images/title_screen.png"), // your big image
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -10.0), // behind UI
                scale: Vec3::splat(1.0),
                ..default()
            },
            ..default()
        },
        MenuBackground, // 🔑 IMPORTANT
    ));
}

fn spawn_snowflakes(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SnowSpawnTimer>,
    state: Res<GameState>,
) {
    // ❄️ No snow in menu
    if *state != GameState::Playing {
        return;
    }

    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-380.0..380.0);
        let size = rng.gen_range(2.0..4.0);
        let speed = rng.gen_range(30.0..80.0);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                transform: Transform::from_xyz(x, 380.0, -1.0),
                ..default()
            },
            Snowflake,
            SnowSpeed(speed),
        ));
    }
}

fn move_snow(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &SnowSpeed), With<Snowflake>>,
    time: Res<Time>,
    state: Res<GameState>,
) {
    if *state != GameState::Playing {
        return;
    }

    for (entity, mut transform, speed) in query.iter_mut() {
        transform.translation.y -= speed.0 * time.delta_seconds();

        // ❄️ When snow reaches bottom → despawn
        if transform.translation.y < -380.0 {
            commands.entity(entity).despawn();
        }
    }
}

/* =======================
   PLAYER MOVEMENT
======================= */

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if *state != GameState::Playing {
        return;
    }

    let mut transform = query.single_mut();
    let speed = 5.0;
    let mut movement = 0.0;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        movement -= speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        movement += speed;
    }

    // ✅ APPLY MOVEMENT ONCE
    transform.translation.x += movement;

    // 🔒 SCREEN BOUNDS
    let min_x = -320.0;
    let max_x = 320.0;
    transform.translation.x = transform.translation.x.clamp(min_x, max_x);
}

fn overload_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut data: ResMut<GameData>,
    state: Res<GameState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if *state != GameState::Playing {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        commands.spawn(AudioBundle {
            source: asset_server.load("audio/Fire.wav"),
            settings: PlaybackSettings::DESPAWN,
        });
    }

    if keyboard.pressed(KeyCode::Space) {
        data.overloading = true;
        data.speed_multiplier = 2.0;
        data.heat += 0.8;
    } else {
        data.overloading = false;
        data.speed_multiplier = 1.0;
        data.heat -= 0.4;
    }

    data.heat = data.heat.clamp(0.0, 100.0);
}
fn spawn_presents(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<GameState>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    difficulty: Res<Difficulty>, // ✅ REQUIRED
) {
    if *state != GameState::Playing {
        return;
    }

    timer.0.tick(time.delta());

    timer.0.set_duration(std::time::Duration::from_secs_f32(
        (1.2_f32 / difficulty.level).clamp(0.3_f32, 1.2_f32),
    ));

    if timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-300.0_f32..300.0_f32);

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites/present.png"),
                transform: Transform {
                    translation: Vec3::new(x, 300.0, 0.0),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            },
            Present,
        ));
    }
}

fn move_presents(
    mut commands: Commands,
    state: Res<GameState>,
    data: Res<GameData>,
    difficulty: Res<Difficulty>,
    time_scale: Res<TimeScale>,
    mut query: Query<(Entity, &mut Transform), With<Present>>,
) {
    if *state != GameState::Playing {
        return;
    }

    if *state == GameState::Crashed {
        return;
    }

    let base_speed = 3.0;
    let speed = base_speed * difficulty.level * data.speed_multiplier;

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.y -= speed * time_scale.value;

        if transform.translation.y < -350.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn collect_presents(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    state: Res<GameState>,
    player: Query<&Transform, With<Player>>,
    presents: Query<(Entity, &Transform), With<Present>>,
    asset_server: Res<AssetServer>, // ✅ REQUIRED for sound
) {
    if *state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, transform) in presents.iter() {
        let distance = player_pos.distance(transform.translation);

        if distance < 40.0 {
            // 🎯 SCORE LOGIC
            if data.overloading {
                data.score += 25; // BONUS
            } else {
                data.score += 10;
            }

            // 🔊 PRESENT COLLECT SOUND (clean & satisfying)
            commands.spawn(AudioBundle {
                source: asset_server.load("audio/PRESENT.wav"),
                settings: PlaybackSettings {
                    volume: Volume::new(0.55), // ✅ balanced volume
                    ..default()
                },
            });

            // 🧹 REMOVE PRESENT
            commands.entity(entity).despawn();
            break; // ✅ prevents double collection in one frame
        }
    }
}

fn crash_check(mut state: ResMut<GameState>, data: Res<GameData>) {
    if *state == GameState::Playing && data.heat >= 100.0 {
        *state = GameState::Crashed;
        println!("💥 SYSTEM FAILURE");
    }
}
fn fade_game_over(
    mut fade: ResMut<GameOverFade>,
    state: Res<GameState>,
    mut query: Query<&mut BackgroundColor, With<GameOverOverlay>>,
    time: Res<Time>,
) {
    if *state != GameState::Crashed {
        fade.alpha = 0.0;
        return;
    }

    fade.alpha = (fade.alpha + time.delta_seconds() * 0.6).clamp(0.0, 0.85);

    if let Ok(mut bg) = query.get_single_mut() {
        bg.0.set_a(fade.alpha);
    }
}

fn spawn_game_over_overlay(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<GameState>,
    query: Query<Entity, With<GameOverOverlay>>,
) {
    if *state != GameState::Crashed || !query.is_empty() {
        return;
    }

    let font = asset_server.load("fonts/PixelOperator8-Bold.ttf");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(18.0), // 🔥 spacing
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
                ..default()
            },
            GameOverOverlay,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SYSTEM FAILURE",
                TextStyle {
                    font: font.clone(),
                    font_size: 42.0,
                    color: Color::RED,
                },
            ));

            parent.spawn(TextBundle::from_section(
                "CHRISTMAS RUINED",
                TextStyle {
                    font: font.clone(),
                    font_size: 28.0,
                    color: Color::ORANGE_RED,
                },
            ));

            parent.spawn(TextBundle::from_section(
                "PRESS R TO REBOOT",
                TextStyle {
                    font,
                    font_size: 20.0,
                    color: Color::GRAY,
                },
            ));
        });
}

/* =======================
   UI SETUP
======================= */

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/PixelOperator8-Bold.ttf");

    // SCORE
    commands.spawn((
        TextBundle::from_section(
            "SCORE: 0",
            TextStyle {
                font: font.clone(),
                font_size: 28.0,
                color: Color::GREEN,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ScoreText,
    ));

    // HEAT
    commands.spawn((
        TextBundle::from_section(
            "HEAT: 0%",
            TextStyle {
                font,
                font_size: 28.0,
                color: Color::RED,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(45.0),
            left: Val::Px(10.0),
            ..default()
        }),
        HeatText,
    ));
}

/* =======================
   UPDATE UI
======================= */

fn update_ui(
    data: Res<GameData>,
    mut texts: ParamSet<(
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<HeatText>>,
    )>,
) {
    if let Ok(mut score_text) = texts.p0().get_single_mut() {
        score_text.sections[0].value = format!("SCORE: {}", data.score);
    }

    if let Ok(mut heat_text) = texts.p1().get_single_mut() {
        heat_text.sections[0].value = format!("HEAT: {}%", data.heat as i32);

        heat_text.sections[0].style.color = if data.heat > 70.0 {
            Color::ORANGE_RED
        } else {
            Color::RED
        };
    }
}

/* =======================
   MUSIC
======================= */

fn play_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("audio/BACKGROUNG_LOOP.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.25), // ✅ soft & clean
                ..default()
            },
        },
        BackgroundMusic,
    ));
}

/* =======================
   GAME RESTART
======================= */

fn restart_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut data: ResMut<GameData>,
    mut state: ResMut<GameState>,
    mut commands: Commands,
    game_over_ui: Query<Entity, With<GameOverOverlay>>,
    mut fade: ResMut<GameOverFade>,
    mut difficulty: ResMut<Difficulty>,
    mut player: Query<&mut Visibility, With<Player>>,
    mut played: ResMut<CrashSoundPlayed>
) {
    if keyboard.just_pressed(KeyCode::KeyR) && *state == GameState::Crashed {
        // 🔄 Reset gameplay data
        data.score = 0;
        data.heat = 0.0;
        data.speed_multiplier = 1.0;
        data.overloading = false;

        // 🔄 Reset difficulty
        difficulty.level = 1.0;
        difficulty.time_alive = 0.0;

        // 🔄 Reset fade
        fade.alpha = 0.0;

        // 🔄 Switch state
        *state = GameState::Playing;

        // 🧹 Remove GAME OVER overlay
        for e in game_over_ui.iter() {
            commands.entity(e).despawn_recursive();
        }

        // 👀 SHOW SANTA AGAIN
        if let Ok(mut vis) = player.get_single_mut() {
            *vis = Visibility::Visible;
        }
        played.0 = false;



        println!("🔁 SYSTEM REBOOTED");
    }
}
