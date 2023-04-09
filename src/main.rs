use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::Rng;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 144.0;

const BACKGROUND_COLOR: Color = Color::rgb(0.3, 0.05, 0.025);
const NANO_COLOR: Color = Color::rgb(1.0, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

const PLAYER_BULLET_DAMAGE: f32 = 12.0;
const PLAYER_BULLET_EFFECT_RISK: i32 = 10;
/// Damage dealt when two cells with different types collide.
const CELL_INTERCOLLISION_DAMAGE: f32 = 8.0;
/// Damage dealt when player touches a cell in no shooting mode.
const PLAYER_COLLISION_DAMAGE: f32 = 12.0;

const SIDE_EFFECT_DURATION: f32 = 16.0;

const BACKGROUND_SCROLL_SPEED: f32 = 200.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .add_event::<SideEffectUpdateEvent>()
        .insert_resource(Boundaries::default())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_startup_system(setup)
        .add_system(setup_game.in_schedule(OnEnter(GameState::Init)))
        .add_system(start_game.in_schedule(OnEnter(GameState::Running)))
        .add_system(change_music.in_schedule(OnEnter(GameState::Running)))
        .add_system(change_music.in_schedule(OnEnter(GameState::Ended)))
        .add_systems(
            (
                spawner_system.run_if(in_state(GameState::Running)),
                player_shoot.run_if(in_state(GameState::Running)),
                scroller_system.run_if(in_state(GameState::Running)),
                physics_objects.run_if(in_state(GameState::Running)),
                cell_despawner.run_if(in_state(GameState::Running)),
                player_bullet_despawner.run_if(in_state(GameState::Running)),
                player_collisions.run_if(in_state(GameState::Running)),
                player_bullet_collisions.run_if(in_state(GameState::Running)),
                cell_cell_collisions
                    .after(physics_objects)
                    .run_if(in_state(GameState::Running)),
                player_movement.run_if(in_state(GameState::Running)),
                side_effect_system.run_if(in_state(GameState::Running)),
                game_over_check.run_if(in_state(GameState::Running)),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .add_system(update_scoreboard.run_if(in_state(GameState::Running)))
        .add_system(update_side_effect_text.run_if(in_state(GameState::Running)))
        .add_system(game_over_system.run_if(in_state(GameState::Ended)))
        .add_system(welcome_system.run_if(in_state(GameState::Init)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Init,
    Running,
    Ended,
}

#[derive(Component)]
struct Player {
    shoot_timer: Timer,
}

#[derive(Component, PartialEq, Eq)]
enum TopText {
    Header,
    Sub,
}

#[derive(Component)]
struct WelcomeText;

#[derive(Component)]
struct Cell {
    target_radius: f32,
    cell_type: CellType,
    top_bound: f32,
    /// How much patient hp will be recovered/lost when this cell reaches the end.
    patient_hp: i32,
}

enum CellType {
    /// This is a cell belonging to the patient's body.
    Body {
        /// How much hp the patient will lose when this cell is destroyed.
        patient_hp: i32,
    },
    /// Enemy cell
    Germ,
}

#[derive(Component)]
struct PlayerBullet;

/// Component related to side effects
#[derive(Component, PartialEq, Eq, Clone, Copy)]
enum SideFx {
    Left,
    Right,
}

/// Tracks score, player health, etc.
#[derive(Resource)]
struct Scoreboard {
    score: usize,
    patient_hp: i32,
}

#[derive(PartialEq, Eq)]
enum SideEffectType {
    None,
    SlowerMovement,
    FasterMovement,
    NoShooting,
    NoKnockback,
}

impl SideEffectType {
    fn random() -> SideEffectType {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => SideEffectType::SlowerMovement,
            1 => SideEffectType::FasterMovement,
            2 => SideEffectType::NoShooting,
            3 => SideEffectType::NoKnockback,
            _ => unreachable!(),
        }
    }

    fn movement_multiplier(&self) -> f32 {
        match self {
            SideEffectType::SlowerMovement => 0.5,
            SideEffectType::FasterMovement => 2.0,
            _ => 1.0,
        }
    }

    fn name(&self) -> &str {
        match self {
            SideEffectType::None => "None",
            SideEffectType::SlowerMovement => "Slower movement",
            SideEffectType::FasterMovement => "Faster movement",
            SideEffectType::NoShooting => "No shooting\nTouch to kill",
            SideEffectType::NoKnockback => "No bullet knockback",
        }
    }
}

enum SideEffectUpdateEvent {
    Left { risk: i32 },
    Right { risk: i32 },
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            score: 0,
            patient_hp: 100,
        }
    }
}

#[derive(Resource)]
struct TextStyles {
    in_game: TextStyle,
    label_style: TextStyle,
}

#[derive(Resource)]
struct SideEffects {
    left_effect_risk: i32,
    right_effect_risk: i32,
    left_effect: SideEffectType,
    right_effect: SideEffectType,
    left_effect_x: f32,
    right_effect_x: f32,
    left_timer: Timer,
    right_timer: Timer,
}

impl Default for SideEffects {
    fn default() -> Self {
        Self {
            left_effect_risk: 0,
            right_effect_risk: 0,
            left_effect: SideEffectType::None,
            right_effect: SideEffectType::None,
            left_effect_x: -100.0,
            right_effect_x: 100.0,
            left_timer: Timer::from_seconds(SIDE_EFFECT_DURATION, TimerMode::Repeating),
            right_timer: Timer::from_seconds(SIDE_EFFECT_DURATION, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
enum ScoreboardText {
    Score,
    PatientHp,
    LeftEffectRisk,
    RightEffectRisk,
}

#[derive(Component)]
struct Physics {
    velocity: Vec2,
    acceleration: Vec2,
    elasticity: f32,
    radius: f32,
}

#[derive(Resource)]
struct Spawner {
    timer: Timer,
    circle_mesh: Mesh2dHandle,
    nano_color: Handle<ColorMaterial>,
    player_texture: Handle<Image>,
    blood_texture: Handle<Image>,
    germ_texture: Handle<Image>,
    vein_side_texture: Handle<Image>,
    vein_bg_texture: Handle<Image>,
}

#[derive(Resource)]
struct Boundaries {
    left_wall: f32,
    right_wall: f32,
    top: f32,
    bottom: f32,
}

#[derive(Component)]
struct Scroller;

impl Default for Boundaries {
    fn default() -> Self {
        Boundaries {
            left_wall: -320.0,
            right_wall: 320.0,
            top: 360.0,
            bottom: -360.0,
        }
    }
}

#[derive(Resource)]
struct MusicResource {
    game_source: Handle<AudioSource>,
    over_source: Handle<AudioSource>,
    current: Option<Handle<AudioSink>>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    let circle_mesh: Mesh2dHandle = meshes.add(shape::Circle::default().into()).into();
    let nano_color = materials.add(ColorMaterial::from(NANO_COLOR));

    commands.insert_resource(Spawner {
        timer: Timer::from_seconds(10.0, TimerMode::Repeating),
        circle_mesh,
        nano_color,
        player_texture: asset_server.load("graphics/player.png"),
        blood_texture: asset_server.load("graphics/bloodcell.png"),
        germ_texture: asset_server.load("graphics/germ.png"),
        vein_side_texture: asset_server.load("graphics/veinside.png"),
        vein_bg_texture: asset_server.load("graphics/veinbg.png"),
    });

    commands.insert_resource(MusicResource {
        game_source: asset_server.load("music/game.ogg"),
        over_source: asset_server.load("music/over.ogg"),
        current: None,
    });

    let label_style = TextStyle {
        font: asset_server.load("fonts/Kanit-Regular.ttf"),
        font_size: 32.0,
        color: TEXT_COLOR,
    };
    let number_style = TextStyle {
        font: asset_server.load("fonts/Kanit-Regular.ttf"),
        font_size: 64.0,
        color: TEXT_COLOR,
    };

    commands.insert_resource(TextStyles {
        in_game: TextStyle {
            font: asset_server.load("fonts/Kanit-Regular.ttf"),
            font_size: 42.0,
            color: TEXT_COLOR,
        },
        label_style: label_style.clone(),
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                // fill the entire window
                size: Size::all(Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect {
                    left: Val::Px(8.0),
                    top: Val::Px(8.0),
                    right: Val::Px(8.0),
                    bottom: Val::Px(8.0),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                "Left Side-Effect Risk",
                label_style.clone(),
            ));
            builder.spawn((
                TextBundle::from_section("0", number_style.clone()),
                ScoreboardText::LeftEffectRisk,
            ));
            builder.spawn(TextBundle::from_section("Score", label_style.clone()));
            builder.spawn((
                TextBundle::from_section("0", number_style.clone()),
                ScoreboardText::Score,
            ));
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                // fill the entire window
                size: Size::all(Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                padding: UiRect {
                    left: Val::Px(8.0),
                    top: Val::Px(8.0),
                    right: Val::Px(8.0),
                    bottom: Val::Px(8.0),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                "Right Side-Effect Risk",
                label_style.clone(),
            ));
            builder.spawn((
                TextBundle::from_section("0", number_style.clone()),
                ScoreboardText::RightEffectRisk,
            ));
            builder.spawn(TextBundle::from_section(
                "Patient Health",
                label_style.clone(),
            ));
            builder.spawn((
                TextBundle::from_section("100", number_style.clone()),
                ScoreboardText::PatientHp,
            ));
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                // fill the entire window
                size: Size::all(Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                // align_self: AlignSelf::Center,
                padding: UiRect {
                    left: Val::Px(8.0),
                    top: Val::Px(8.0),
                    right: Val::Px(8.0),
                    bottom: Val::Px(8.0),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section("", number_style.clone()),
                TopText::Header,
            ));
            builder.spawn((
                TextBundle::from_section("", label_style.clone()),
                TopText::Sub,
            ));
        });
}

/// Add the game's entities
fn setup_game(
    mut commands: Commands,
    spawner: Res<Spawner>,
    query: Query<Entity, Or<(With<Physics>, With<SideFx>, With<Scroller>)>>,
    mut top_text_query: Query<(&mut Text, &TopText)>,
    text_styles: Res<TextStyles>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }

    for (mut text, top_text) in &mut top_text_query {
        if *top_text == TopText::Sub {
            text.sections[0].value = "Press ENTER to start.".to_owned();
        } else {
            text.sections[0].value = "Side Vein Effect".to_owned();
        }
    }

    for i in 0..4 {
        let x_mul: f32 = if i % 2 == 0 { 1.0 } else { -1.0 };
        let y: f32 = if i >= 2 { 1080.0 } else { -360.0 };
        let transform = Transform {
            translation: vec3(x_mul * 640.0, y, 0.0),
            ..default()
        };
        commands.spawn((
            SpriteBundle {
                transform,
                texture: spawner.vein_side_texture.clone(),
                ..default()
            },
            Scroller,
        ));
    }

    for i in 0..2 {
        let y: f32 = if i > 0 { 1080.0 } else { -360.0 };
        let transform = Transform {
            translation: vec3(0.0, y, 0.0),
            ..default()
        };
        commands.spawn((
            SpriteBundle {
                transform,
                texture: spawner.vein_bg_texture.clone(),
                ..default()
            },
            Scroller,
        ));
    }

    // PLAYER
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            texture: spawner.player_texture.clone(),
            ..default()
        },
        Physics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            elasticity: 0.5,
            radius: 15.0,
        },
        Player {
            shoot_timer: Timer::from_seconds(0.25, TimerMode::Once),
        },
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Percent(60.0),
                        bottom: Val::Percent(100.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        left: Val::Px(8.0),
                        top: Val::Px(8.0),
                        right: Val::Px(8.0),
                        bottom: Val::Px(8.0),
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            WelcomeText,
        ))
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section("CONTROLS", text_styles.label_style.clone()),
                WelcomeText,
            ));
            builder.spawn((
                TextBundle::from_section("Arrow keys: Move", text_styles.label_style.clone()),
                WelcomeText,
            ));
            builder.spawn((
                TextBundle::from_section("Space or A: Shoot", text_styles.label_style.clone()),
                WelcomeText,
            ));
            builder.spawn((
                TextBundle::from_section("GAMEPLAY", text_styles.label_style.clone()),
                WelcomeText,
            ));
            builder.spawn((
                TextBundle::from_section(
                    "Let the blood cells (red cells) pass. Shoot germs (green cells).",
                    text_styles.label_style.clone(),
                ),
                WelcomeText,
            ));
            builder.spawn((
                TextBundle::from_section(
                    "If you miss shots, side-effect risk will increase.",
                    text_styles.label_style.clone(),
                ),
                WelcomeText,
            ));
        });

    commands.insert_resource(Scoreboard::default());
    commands.insert_resource(SideEffects::default());
}

fn start_game(
    mut commands: Commands,
    query: Query<Entity, With<WelcomeText>>,
    mut top_text_query: Query<&mut Text, With<TopText>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    for mut text in &mut top_text_query {
        text.sections[0].value = "".to_owned();
    }
}

fn change_music(
    mut music_res: ResMut<MusicResource>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
    game_state: Res<State<GameState>>,
) {
    if let Some(music) = music_res.current.take() {
        if let Some(music) = audio_sinks.get(&music) {
            music.stop();
        }
    }
    let new_music = match game_state.0 {
        GameState::Running => music_res.game_source.clone(),
        GameState::Ended => music_res.over_source.clone(),
        _ => {
            return;
        }
    };
    let sink = audio.play_with_settings(new_music, PlaybackSettings::LOOP);
    let sink = audio_sinks.get_handle(sink);
    music_res.current = Some(sink);
}

fn scroller_system(mut query: Query<(&mut Transform, &Scroller)>) {
    for (mut transform, _wall) in &mut query {
        transform.translation.y -= BACKGROUND_SCROLL_SPEED * TIME_STEP;
        if transform.translation.y <= -1260.0 {
            transform.translation.y = 1620.0;
        }
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    boundaries: Res<Boundaries>,
    mut query: Query<(&mut Transform, &mut Physics), With<Player>>,
) {
    let (mut transform, mut physics) = query.single_mut();
    let mut acceleration = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::Left) {
        acceleration.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        acceleration.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        acceleration.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        acceleration.y -= 1.0;
    }
    acceleration = acceleration.normalize_or_zero();
    acceleration.x *= 700.0;
    acceleration.y *= 700.0;

    physics.acceleration = acceleration;

    let top_bound = boundaries.top - physics.radius;
    let bottom_bound = boundaries.bottom + physics.radius;

    if transform.translation.y < bottom_bound {
        transform.translation.y = bottom_bound;
        physics.acceleration.y = physics.acceleration.y.max(0.0);
        physics.velocity.y = physics.velocity.y.max(0.0);
    } else if transform.translation.y > top_bound {
        transform.translation.y = top_bound;
        physics.acceleration.y = physics.acceleration.y.min(0.0);
        physics.velocity.y = physics.velocity.y.min(0.0);
    }
}

fn player_shoot(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut Player)>,
    spawner: Res<Spawner>,
    side_effects: Res<SideEffects>,
) {
    let (transform, mut player) = query.single_mut();
    if !(player.shoot_timer.tick(time.delta()).finished()
        && (keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Space)))
    {
        return;
    }
    if transform.translation.x > side_effects.right_effect_x
        && side_effects.right_effect == SideEffectType::NoShooting
    {
        return;
    } else if transform.translation.x < side_effects.left_effect_x
        && side_effects.left_effect == SideEffectType::NoShooting
    {
        return;
    }
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: spawner.circle_mesh.clone(),
            material: spawner.nano_color.clone(),
            transform: Transform::from_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                1.0,
            ))
            .with_scale(Vec3::new(8.0, 8.0, 8.0)),
            ..default()
        },
        Physics {
            velocity: vec2(0.0, 600.0),
            acceleration: vec2(0.0, 0.0),
            elasticity: 0.9,
            radius: 4.0,
        },
        PlayerBullet,
    ));
    player.shoot_timer.reset();
}

fn update_scoreboard(
    scoreboard: Res<Scoreboard>,
    side_effects: Res<SideEffects>,
    mut query: Query<(&mut Text, &ScoreboardText)>,
) {
    for (mut text, text_type) in &mut query {
        text.sections[0].value = match text_type {
            ScoreboardText::Score => scoreboard.score.to_string(),
            ScoreboardText::PatientHp => format!("{}%", scoreboard.patient_hp),
            ScoreboardText::LeftEffectRisk => format!("{}%", side_effects.left_effect_risk),
            ScoreboardText::RightEffectRisk => format!("{}%", side_effects.right_effect_risk),
        }
    }
}

fn update_side_effect_text(side_effects: Res<SideEffects>, mut query: Query<(&mut Text, &SideFx)>) {
    for (mut text, fx) in &mut query {
        text.sections[0].value = match fx {
            SideFx::Left => format!(
                "{}\n{:.0} seconds",
                side_effects.left_effect.name(),
                side_effects.left_timer.remaining_secs().ceil()
            ),
            SideFx::Right => format!(
                "{}\n{:.0} seconds",
                side_effects.right_effect.name(),
                side_effects.right_timer.remaining_secs().ceil()
            ),
        }
    }
}

/// Player-Cell collisions.
fn player_collisions(
    mut player_query: Query<(&mut Transform, &mut Physics), With<Player>>,
    mut cell_query: Query<(&mut Transform, &mut Physics, &mut Cell), Without<Player>>,
    side_effects: Res<SideEffects>,
) {
    let (mut player_transform, mut player_physics) = player_query.single_mut();

    for (mut transform, mut physics, mut cell) in &mut cell_query {
        if elastic_collision(
            &mut player_transform,
            &mut player_physics,
            &mut transform,
            &mut physics,
        ) {
            if (player_transform.translation.x > side_effects.right_effect_x
                && side_effects.right_effect == SideEffectType::NoShooting)
                || (player_transform.translation.x < side_effects.left_effect_x
                    && side_effects.left_effect == SideEffectType::NoShooting)
            {
                // Damage the cells by touching in no shooting mode
                cell.target_radius -= PLAYER_COLLISION_DAMAGE;
            }
        }
    }
}

/// Player bullet and Cell collisions.
fn player_bullet_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    bullet_query: Query<(Entity, &Transform, &PlayerBullet)>,
    mut cell_query: Query<(&Transform, &mut Physics, &mut Cell)>,
    side_effects: Res<SideEffects>,
) {
    for (bullet_entity, bullet_transform, _bullet) in &bullet_query {
        let bullet_radius: f32 = 4.0;
        for (cell_transform, mut cell_physics, mut cell) in &mut cell_query {
            let dp = cell_transform.translation - bullet_transform.translation;
            let dist = (dp.x * dp.x) + (dp.y * dp.y);
            let total_radius = bullet_radius + cell_physics.radius;
            let rad2 = total_radius * total_radius;
            if dist <= rad2 {
                commands.entity(bullet_entity).despawn();
                cell.target_radius -= PLAYER_BULLET_DAMAGE;
                if !((bullet_transform.translation.x > side_effects.right_effect_x
                    && side_effects.right_effect == SideEffectType::NoKnockback)
                    || (bullet_transform.translation.x < side_effects.left_effect_x
                        && side_effects.left_effect == SideEffectType::NoKnockback))
                {
                    cell_physics.velocity.y += 200.0;
                    cell_physics.acceleration.y -= 50.0;
                }

                if let CellType::Germ = cell.cell_type {
                    scoreboard.score += 1;
                }
            }
        }
    }
}

fn vec_along(a: Vec2, b: Vec2) -> (Vec2, Vec2) {
    let along = b * a.dot(b);
    let not_along = a - along;
    (along, not_along)
}

fn elastic_collision(
    t1: &mut Transform,
    p1: &mut Physics,
    t2: &mut Transform,
    p2: &mut Physics,
) -> bool {
    let diff: Vec2 = (t1.translation - t2.translation).truncate();
    let total_radius = p1.radius + p2.radius;
    if diff.length_squared() > total_radius * total_radius {
        return false;
    }

    // Assume densities are the same: mass is proportional to size.
    let m1 = p1.radius;
    let m2 = p2.radius;

    // Solve velocity
    let normal = diff.normalize_or_zero();
    let (v1, w1) = vec_along(p1.velocity, normal);
    let (v2, w2) = vec_along(p2.velocity, normal);
    // v1i + v1f = v2i + v2f
    // v1f = v2i + v2f - v1i
    // Conservation of momentum
    // m1 v1i + m2 v2i = m1 v1f + m2 v2f
    // m1 v1i + m2 v2i = m1 (v2i + v2f - v1i) + m2 v2f
    // m1 (v1i - v2i + v1i) + m2 v2i = m1 v2f + m2 v2f
    // v2f = (m1 (v1i - v2i + v1i) + m2 v2i) / (m1 + m2)
    let v2f = (m1 * (v1 + v1 - v2) + m2 * v2) / (m1 + m2);
    let v1f = v2 + v2f - v1;
    p1.velocity = v1f + w1;
    p2.velocity = v2f + w2;

    // Solve position
    let push_length = (diff.length() - total_radius) * 0.6;
    let push_x = normal.x * push_length;
    let push_y = normal.y * push_length;
    t1.translation.x -= push_x;
    t1.translation.y -= push_y;
    t2.translation.x += push_x;
    t2.translation.y += push_y;

    true
}

fn cell_cell_collisions(mut query: Query<(&mut Transform, &mut Physics, &mut Cell)>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut t1, mut p1, mut c1), (mut t2, mut p2, mut c2)]) = combinations.fetch_next()
    {
        if !elastic_collision(&mut t1, &mut p1, &mut t2, &mut p2) {
            continue;
        }
        if std::mem::discriminant(&c1.cell_type) != std::mem::discriminant(&c2.cell_type) {
            // Cells have different types
            c1.target_radius -= CELL_INTERCOLLISION_DAMAGE;
            c2.target_radius -= CELL_INTERCOLLISION_DAMAGE;
        }
    }
}

fn game_over_check(
    scoreboard: Res<Scoreboard>,
    mut query: Query<(&mut Text, &TopText)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if scoreboard.patient_hp <= 0 {
        for (mut text, text_type) in &mut query {
            text.sections[0].value = match text_type {
                TopText::Header => "GAME OVER".to_owned(),
                TopText::Sub => "PRESS R TO RESTART".to_owned(),
            };
        }
        next_state.set(GameState::Ended);
    }
}

fn game_over_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        next_state.set(GameState::Init);
    }
}

fn welcome_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        next_state.set(GameState::Running);
    }
}

/// Update physics objects.
fn physics_objects(
    boundaries: Res<Boundaries>,
    side_effects: Res<SideEffects>,
    mut query: Query<(&mut Transform, &mut Physics)>,
) {
    for (mut transform, mut physics) in &mut query {
        let mut velocity_mul = TIME_STEP;
        if transform.translation.x > side_effects.right_effect_x {
            velocity_mul *= side_effects.right_effect.movement_multiplier();
        } else if transform.translation.x < side_effects.left_effect_x {
            velocity_mul *= side_effects.left_effect.movement_multiplier();
        }
        physics.velocity.x += physics.acceleration.x * TIME_STEP;
        physics.velocity.y += physics.acceleration.y * TIME_STEP;
        transform.translation.x += physics.velocity.x * velocity_mul;
        transform.translation.y += physics.velocity.y * velocity_mul;

        let radius = physics.radius;
        if transform.translation.x - radius < boundaries.left_wall {
            transform.translation.x = boundaries.left_wall + radius;
            physics.velocity.x *= -physics.elasticity;
        } else if transform.translation.x + radius > boundaries.right_wall {
            transform.translation.x = boundaries.right_wall - radius;
            physics.velocity.x *= -physics.elasticity;
        }
    }
}

fn cell_despawner(
    mut commands: Commands,
    boundaries: Res<Boundaries>,
    mut query: Query<(Entity, &mut Transform, &mut Physics, &mut Cell)>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for (entity, mut transform, mut physics, mut cell) in &mut query {
        let scale_diff = cell.target_radius - physics.radius;
        let scale_speed = TIME_STEP * 100.0;
        if scale_diff.abs() > scale_speed {
            physics.radius += scale_diff.signum() * scale_speed;
        } else {
            physics.radius = cell.target_radius;
        }
        transform.scale.x = physics.radius / 45.0;
        transform.scale.y = physics.radius / 45.0;
        if physics.radius < 16.0 {
            cell.target_radius = 0.0;
        }
        if physics.radius < 5.0 {
            commands.entity(entity).despawn();
            match cell.cell_type {
                CellType::Body { patient_hp } => {
                    scoreboard.patient_hp -= patient_hp;
                }
                CellType::Germ => {
                    scoreboard.score += 1;
                }
            }
        } else if transform.translation.y + physics.radius < boundaries.bottom {
            commands.entity(entity).despawn();
            scoreboard.patient_hp +=
                (cell.patient_hp as f32 * ((physics.radius + 5.0) / 50.0)).ceil() as i32;
            scoreboard.patient_hp = scoreboard.patient_hp.min(100);
        } else {
            // Don't let cells go outside the screen from the top
            if transform.translation.y + physics.radius > cell.top_bound {
                transform.translation.y = cell.top_bound - physics.radius;
            }
            cell.top_bound = boundaries
                .top
                .max(transform.translation.y + physics.radius - TIME_STEP * 100.0);
        }
    }
}

fn player_bullet_despawner(
    mut commands: Commands,
    boundaries: Res<Boundaries>,
    query: Query<(Entity, &Transform, &PlayerBullet)>,
    mut side_effects: ResMut<SideEffects>,
    mut side_effect_events: EventWriter<SideEffectUpdateEvent>,
) {
    for (entity, transform, _) in &query {
        // Allow some buffer space (cells can momentarily go outside screen)
        if transform.translation.y > boundaries.top + 120.0 {
            commands.entity(entity).despawn();
            if transform.translation.x > 0.0 {
                let risk = side_effects.right_effect_risk;
                side_effect_events.send(SideEffectUpdateEvent::Right { risk });
                side_effects.right_effect_risk += PLAYER_BULLET_EFFECT_RISK;
            } else {
                let risk = side_effects.left_effect_risk;
                side_effect_events.send(SideEffectUpdateEvent::Left { risk });
                side_effects.left_effect_risk += PLAYER_BULLET_EFFECT_RISK;
            }
        }
    }
}

fn spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    boundaries: Res<Boundaries>,
    mut spawner: ResMut<Spawner>,
    cell_query: Query<&Cell>,
    scoreboard: Res<Scoreboard>,
) {
    if !(cell_query.is_empty() || spawner.timer.tick(time.delta()).just_finished()) {
        return;
    }
    spawner.timer.reset();
    let mut rng = rand::thread_rng();
    let range_x = boundaries.right_wall - boundaries.left_wall;
    let min_enemies = 2 + (scoreboard.score / 90).clamp(0, 4);
    let max_enemies = min_enemies + 3;
    let count = rng.gen_range(min_enemies..max_enemies);
    let x_vel_randomness = 75.0 + (scoreboard.score as f32 / 2.0).clamp(5.0, 125.0);
    let y_vel_base = -(scoreboard.score as f32 / 2.5).clamp(10.0, 200.0);
    for i in 0..count {
        let radius = 45.0;
        let min_x = boundaries.left_wall + radius;
        let range_x = range_x - radius * 2.0;
        let translation = Vec3::new(
            rng.gen_range(0.0..range_x) + min_x,
            boundaries.top + radius + radius * 2.0 * i as f32,
            1.0,
        );
        let velocity = vec2(
            rng.gen_range(-x_vel_randomness..x_vel_randomness),
            y_vel_base - rng.gen_range(0.0..100.0),
        );
        let (cell, texture) = if rng.gen_bool(0.5) {
            (
                Cell {
                    top_bound: translation.y + radius,
                    cell_type: CellType::Body { patient_hp: 10 },
                    target_radius: radius,
                    patient_hp: 1,
                },
                spawner.blood_texture.clone(),
            )
        } else {
            (
                Cell {
                    top_bound: translation.y + radius,
                    cell_type: CellType::Germ,
                    target_radius: radius,
                    patient_hp: -10,
                },
                spawner.germ_texture.clone(),
            )
        };
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(translation),
                texture,
                ..default()
            },
            Physics {
                velocity,
                acceleration: vec2(0.0, -25.0),
                elasticity: 0.9,
                radius,
            },
            cell,
        ));
    }
}

fn side_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    boundaries: Res<Boundaries>,
    text_styles: Res<TextStyles>,
    mut side_effects: ResMut<SideEffects>,
    query: Query<(Entity, &SideFx)>,
    mut side_effect_events: EventReader<SideEffectUpdateEvent>,
) {
    let spawn_side_effect = |commands: &mut Commands,
                             fx_component: SideFx,
                             effect_x: f32,
                             wall_x: f32,
                             effect: &SideEffectType| {
        let translation = vec2((effect_x + wall_x) / 2.0, 0.0);
        let size = vec2((effect_x - wall_x).abs(), 720.0);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.125, 0.5, 0.5, 0.125),
                    ..default()
                },
                transform: Transform {
                    translation: translation.extend(0.5),
                    scale: size.extend(1.0),
                    ..default()
                },
                ..default()
            },
            fx_component,
        ));
        commands.spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        effect.name().to_owned(),
                        text_styles.in_game.clone(),
                    )],
                    alignment: TextAlignment::Center,
                    ..Default::default()
                },
                text_2d_bounds: bevy::text::Text2dBounds { size },
                transform: Transform {
                    translation: translation.extend(0.625),
                    ..default()
                },
                ..default()
            },
            fx_component,
        ));
    };
    // Determine the side effect risk from events
    let mut left_risk: Option<i32> = None;
    let mut right_risk: Option<i32> = None;
    for event in side_effect_events.iter() {
        match *event {
            SideEffectUpdateEvent::Left { risk } => left_risk = Some(risk),
            SideEffectUpdateEvent::Right { risk } => right_risk = Some(risk),
        }
    }
    // Update existing side effects
    if side_effects.left_effect != SideEffectType::None {
        if side_effects.left_timer.tick(time.delta()).just_finished() {
            side_effects.left_timer.reset();
            side_effects.left_effect = SideEffectType::None;
            for (entity, fx) in &query {
                if *fx == SideFx::Left {
                    commands.entity(entity).despawn();
                }
            }
            if side_effects.left_effect_risk >= 100 {
                left_risk = Some(100);
            }
        }
    }
    if side_effects.right_effect != SideEffectType::None {
        if side_effects.right_timer.tick(time.delta()).just_finished() {
            side_effects.right_timer.reset();
            side_effects.right_effect = SideEffectType::None;
            for (entity, fx) in &query {
                if *fx == SideFx::Right {
                    commands.entity(entity).despawn();
                }
            }
            if side_effects.right_effect_risk >= 100 {
                right_risk = Some(100);
            }
        }
    }
    // Create new side effects
    let mut rng = rand::thread_rng();
    if side_effects.left_effect == SideEffectType::None {
        if let Some(risk) = left_risk {
            if rng.gen_range(0..100) < risk {
                side_effects.left_effect_risk -= 100;
                side_effects.left_effect_risk = side_effects.left_effect_risk.max(0);
                side_effects.left_effect = SideEffectType::random();
                spawn_side_effect(
                    &mut commands,
                    SideFx::Left,
                    side_effects.left_effect_x,
                    boundaries.left_wall,
                    &side_effects.left_effect,
                );
            }
        }
    }
    if side_effects.right_effect == SideEffectType::None {
        if let Some(risk) = right_risk {
            if rng.gen_range(0..100) < risk {
                side_effects.right_effect_risk -= 100;
                side_effects.right_effect_risk = side_effects.right_effect_risk.max(0);
                side_effects.right_effect = SideEffectType::random();
                spawn_side_effect(
                    &mut commands,
                    SideFx::Right,
                    side_effects.right_effect_x,
                    boundaries.right_wall,
                    &side_effects.right_effect,
                );
            }
        }
    }
}
