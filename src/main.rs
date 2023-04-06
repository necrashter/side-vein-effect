use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 144.0;

const B0_COLOR: Color = Color::rgb(61.0 / 255.0, 23.0 / 255.0, 102.0 / 255.0);
const F0_COLOR: Color = Color::rgb(111.0 / 255.0, 26.0 / 255.0, 182.0 / 255.0);
const F1_COLOR: Color = Color::rgb(1.0, 0.0, 50.0 / 255.0);
const GERM_COLOR: Color = Color::rgb(0.05, 0.75, 0.05);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .insert_resource(Scoreboard::default())
        .insert_resource(Boundaries::default())
        .insert_resource(ClearColor(B0_COLOR))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_startup_system(setup)
        .add_systems(
            (
                spawner_system.run_if(in_state(GameState::Running)),
                wall_system.run_if(in_state(GameState::Running)),
                physics_objects.run_if(in_state(GameState::Running)),
                cell_despawner.run_if(in_state(GameState::Running)),
                player_collisions.run_if(in_state(GameState::Running)),
                player_movement.run_if(in_state(GameState::Running)),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .add_system(update_scoreboard)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Running,
    Paused,
    Ended,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct TopText;

#[derive(Component)]
enum Cell {
    /// This is a cell belonging to the patient's body.
    Body {
        /// How much patient hp will be lost when this cell dies.
        patient_hp: i32,
        /// How much hp the player will gain by killing this cell.
        player_hp: i32,
    },
    /// Enemy cell
    Germ {
        /// How much damage this will make to player/patient
        damage: i32,
    },
}

/// Tracks score, player health, etc.
#[derive(Resource)]
struct Scoreboard {
    score: usize,
    player_hp: i32,
    patient_hp: i32,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            score: 0,
            player_hp: 100,
            patient_hp: 100,
        }
    }
}

#[derive(Component)]
enum ScoreboardText {
    Score,
    PlayerHp,
    PatientHp,
}

#[derive(Component)]
struct Physics {
    velocity: Vec2,
    acceleration: Vec2,
    elasticity: f32,
}

#[derive(Resource)]
struct Spawner {
    timer: Timer,
    circle_mesh: Mesh2dHandle,
    body_color: Handle<ColorMaterial>,
    germ_color: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct Boundaries {
    left_wall: f32,
    right_wall: f32,
    top: f32,
    bottom: f32,
}

#[derive(Component)]
struct Wall {
    on_right: bool,
    offset: f32,
}

impl Default for Boundaries {
    fn default() -> Self {
        Boundaries {
            left_wall: 0.0,
            right_wall: 0.0,
            top: 360.0,
            bottom: -360.0,
        }
    }
}

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    mut boundaries: ResMut<Boundaries>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    let circle_mesh: Mesh2dHandle = meshes.add(shape::Circle::default().into()).into();
    let body_color = materials.add(ColorMaterial::from(F0_COLOR));
    let germ_color = materials.add(ColorMaterial::from(GERM_COLOR));

    commands.insert_resource(Spawner {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        circle_mesh,
        body_color,
        germ_color,
    });

    for i in 0..2 {
        let on_right = i > 0;
        let x_mul: f32 = if on_right { 1.0 } else { -1.0 };
        let transform = Transform {
            translation: vec3(x_mul * 640.0, 0.0, 0.0),
            scale: vec3(640.0, 720.0, 1.0),
            ..default()
        };
        if on_right {
            boundaries.right_wall = transform.translation.x - (transform.scale.x / 2.0);
        } else {
            boundaries.left_wall = transform.translation.x + (transform.scale.x / 2.0);
        }
        let offset = transform.scale.x / 2.0 * x_mul;
        commands.spawn((
            SpriteBundle {
                transform,
                sprite: Sprite {
                    color: F0_COLOR,
                    ..default()
                },
                ..default()
            },
            Wall { on_right, offset },
        ));
    }

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(F1_COLOR)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::new(30.0, 30.0, 0.0)),
            ..default()
        },
        Physics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            elasticity: 0.5,
        },
        Player,
    ));

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
                "Nanomachine Health",
                label_style.clone(),
            ));
            builder.spawn((
                TextBundle::from_section("100", number_style.clone()),
                ScoreboardText::PlayerHp,
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
            builder.spawn((TextBundle::from_section("", number_style.clone()), TopText));
        });
}

fn wall_system(boundaries: Res<Boundaries>, mut query: Query<(&mut Transform, &Wall)>) {
    for (mut transform, wall) in &mut query {
        if wall.on_right {
            transform.translation.x = boundaries.right_wall + wall.offset;
        } else {
            transform.translation.x = boundaries.left_wall + wall.offset;
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
    acceleration.x *= 500.0;
    acceleration.y *= 500.0;

    physics.acceleration = acceleration;

    let radius = transform.scale.x / 2.0;
    let top_bound = boundaries.top - radius;
    let bottom_bound = boundaries.bottom + radius;

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

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<(&mut Text, &ScoreboardText)>) {
    for (mut text, text_type) in &mut query {
        text.sections[0].value = match text_type {
            ScoreboardText::Score => scoreboard.score.to_string(),
            ScoreboardText::PlayerHp => scoreboard.player_hp.to_string(),
            ScoreboardText::PatientHp => scoreboard.patient_hp.to_string(),
        }
    }
}

/// Player-Cell collisions.
fn player_collisions(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut player_query: Query<&Transform, With<Player>>,
    cell_query: Query<(Entity, &Transform, &Cell)>,
    mut top_text_query: Query<&mut Text, With<TopText>>,
) {
    let player_transform = player_query.single_mut();
    let player_size = player_transform.scale.y;

    // check collision with walls
    for (entity, transform, cell) in &cell_query {
        let dp = transform.translation - player_transform.translation;
        let dist = (dp.x * dp.x) + (dp.y * dp.y);
        let total_radius = (player_size + transform.scale.y) / 2.0;
        let rad2 = total_radius * total_radius;
        if dist <= rad2 {
            commands.entity(entity).despawn();

            match cell {
                Cell::Body {
                    patient_hp,
                    player_hp,
                } => {
                    scoreboard.player_hp += player_hp;
                    scoreboard.player_hp = scoreboard.player_hp.min(100);
                    scoreboard.patient_hp -= patient_hp;
                    if scoreboard.patient_hp <= 0 {
                        let mut top_text = top_text_query.single_mut();
                        top_text.sections[0].value = "GAME OVER".to_owned();
                        next_state.set(GameState::Ended);
                    }
                }
                Cell::Germ { damage } => {
                    scoreboard.player_hp -= damage;
                    if scoreboard.player_hp <= 0 {
                        let mut top_text = top_text_query.single_mut();
                        top_text.sections[0].value = "GAME OVER".to_owned();
                        next_state.set(GameState::Ended);
                    }
                }
            }
        }
    }
}

/// Update physics objects.
fn physics_objects(boundaries: Res<Boundaries>, mut query: Query<(&mut Transform, &mut Physics)>) {
    for (mut transform, mut physics) in &mut query {
        physics.velocity.x += physics.acceleration.x * TIME_STEP;
        physics.velocity.y += physics.acceleration.y * TIME_STEP;
        transform.translation.x += physics.velocity.x * TIME_STEP;
        transform.translation.y += physics.velocity.y * TIME_STEP;

        let radius = transform.scale.x / 2.0;
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
    query: Query<(Entity, &Transform, &Cell)>,
    mut scoreboard: ResMut<Scoreboard>,
    mut top_text_query: Query<&mut Text, With<TopText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, transform, cell) in &query {
        let radius = transform.scale.x / 2.0;
        if transform.translation.y + radius < boundaries.bottom {
            commands.entity(entity).despawn();
            if let Cell::Germ { damage } = cell {
                scoreboard.patient_hp -= damage;
                if scoreboard.patient_hp <= 0 {
                    let mut top_text = top_text_query.single_mut();
                    top_text.sections[0].value = "GAME OVER".to_owned();
                    next_state.set(GameState::Ended);
                }
            }
        }
    }
}

fn spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    boundaries: Res<Boundaries>,
    mut spawner: ResMut<Spawner>,
) {
    if spawner.timer.tick(time.delta()).just_finished() {
        let range_x = boundaries.right_wall - boundaries.left_wall;
        for i in 0..2 {
            let radius = 45.0;
            let scale = radius * 2.0;
            let min_x = boundaries.left_wall + radius;
            let range_x = range_x - scale;
            let (cell, material) = if rand::random::<bool>() {
                (
                    Cell::Body {
                        patient_hp: 10,
                        player_hp: 5,
                    },
                    spawner.body_color.clone(),
                )
            } else {
                (Cell::Germ { damage: 5 }, spawner.germ_color.clone())
            };
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: spawner.circle_mesh.clone(),
                    material,
                    transform: Transform::from_translation(Vec3::new(
                        rand::random::<f32>() * range_x + min_x,
                        360.0 + radius,
                        1.0,
                    ))
                    .with_scale(Vec3::new(scale, scale, scale)),
                    ..default()
                },
                Physics {
                    velocity: vec2(0.0, -300.0),
                    acceleration: vec2(0.0, 0.0),
                    elasticity: 0.9,
                },
                cell,
            ));
        }
    }
}
