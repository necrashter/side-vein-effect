use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::Rng;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 144.0;

const B0_COLOR: Color = Color::rgb(61.0 / 255.0, 23.0 / 255.0, 102.0 / 255.0);
const F0_COLOR: Color = Color::rgb(111.0 / 255.0, 26.0 / 255.0, 182.0 / 255.0);
const F1_COLOR: Color = Color::rgb(1.0, 0.0, 50.0 / 255.0);
const GERM_COLOR: Color = Color::rgb(0.05, 0.75, 0.05);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

const PLAYER_BULLET_DAMAGE: f32 = 30.0;
/// Damage dealt when two cells with different types collide.
const CELL_INTERCOLLISION_DAMAGE: f32 = 20.0;

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
                player_shoot.run_if(in_state(GameState::Running)),
                wall_system.run_if(in_state(GameState::Running)),
                physics_objects.run_if(in_state(GameState::Running)),
                cell_despawner.run_if(in_state(GameState::Running)),
                player_bullet_despawner.run_if(in_state(GameState::Running)),
                player_collisions.run_if(in_state(GameState::Running)),
                player_bullet_collisions.run_if(in_state(GameState::Running)),
                cell_cell_collisions
                    .after(physics_objects)
                    .run_if(in_state(GameState::Running)),
                player_movement.run_if(in_state(GameState::Running)),
                game_over_check.run_if(in_state(GameState::Running)),
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
struct Player {
    shoot_timer: Timer,
}

#[derive(Component)]
struct TopText;

#[derive(Component)]
struct Cell {
    target_scale: f32,
    cell_type: CellType,
    /// How much patient hp will be recovered/lost when this cell reaches the end.
    patient_hp: i32,
}

enum CellType {
    /// This is a cell belonging to the patient's body.
    Body {
        /// How much hp the player will gain by eating this cell.
        player_hp: i32,
    },
    /// Enemy cell
    Germ {
        /// How much damage this will deal to the player
        damage: i32,
    },
}

#[derive(Component)]
struct PlayerBullet;

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
    nano_color: Handle<ColorMaterial>,
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
    let nano_color = materials.add(ColorMaterial::from(F1_COLOR));

    commands.insert_resource(Spawner {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        circle_mesh,
        body_color,
        germ_color,
        nano_color: nano_color.clone(),
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

    // PLAYER
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: nano_color,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::new(30.0, 30.0, 0.0)),
            ..default()
        },
        Physics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            elasticity: 0.5,
        },
        Player {
            shoot_timer: Timer::from_seconds(0.25, TimerMode::Once),
        },
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
    acceleration.x *= 700.0;
    acceleration.y *= 700.0;

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

fn player_shoot(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut Player)>,
    spawner: Res<Spawner>,
) {
    let (transform, mut player) = query.single_mut();
    if player.shoot_timer.tick(time.delta()).finished() && keyboard_input.pressed(KeyCode::A) {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: spawner.circle_mesh.clone(),
                material: spawner.nano_color.clone(),
                transform: Transform::from_translation(Vec3::new(
                    transform.translation.x,
                    transform.translation.y,
                    0.5,
                ))
                .with_scale(Vec3::new(8.0, 8.0, 8.0)),
                ..default()
            },
            Physics {
                velocity: vec2(0.0, 600.0),
                acceleration: vec2(0.0, 0.0),
                elasticity: 0.9,
            },
            PlayerBullet,
        ));
        player.shoot_timer.reset();
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
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut player_query: Query<&Transform, With<Player>>,
    cell_query: Query<(Entity, &Transform, &Cell)>,
) {
    let player_transform = player_query.single_mut();
    let player_size = player_transform.scale.y;

    for (entity, transform, cell) in &cell_query {
        let dp = transform.translation - player_transform.translation;
        let dist = (dp.x * dp.x) + (dp.y * dp.y);
        let total_radius = (player_size + transform.scale.y) / 2.0;
        let rad2 = total_radius * total_radius;
        if dist <= rad2 {
            commands.entity(entity).despawn();

            match cell.cell_type {
                CellType::Body { player_hp } => {
                    scoreboard.player_hp += player_hp;
                    scoreboard.player_hp = scoreboard.player_hp.min(100);
                    scoreboard.patient_hp -= player_hp;
                }
                CellType::Germ { damage } => {
                    scoreboard.player_hp -= damage;
                    scoreboard.score += 1;
                }
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
) {
    for (bullet_entity, bullet_transform, _bullet) in &bullet_query {
        let bullet_size = bullet_transform.scale.x;
        for (cell_transform, mut cell_physics, mut cell) in &mut cell_query {
            let dp = cell_transform.translation - bullet_transform.translation;
            let dist = (dp.x * dp.x) + (dp.y * dp.y);
            let total_radius = (bullet_size + cell_transform.scale.y) / 2.0;
            let rad2 = total_radius * total_radius;
            if dist <= rad2 {
                commands.entity(bullet_entity).despawn();
                cell.target_scale -= PLAYER_BULLET_DAMAGE;
                cell_physics.velocity.y += 200.0;
                cell_physics.acceleration.y -= 50.0;

                if let CellType::Germ { damage: _ } = cell.cell_type {
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

fn cell_cell_collisions(mut query: Query<(&mut Transform, &mut Physics, &mut Cell)>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut t1, mut p1, mut c1), (mut t2, mut p2, mut c2)]) = combinations.fetch_next()
    {
        let diff: Vec2 = (t1.translation - t2.translation).truncate();
        let total_radius = (t1.scale.x + t2.scale.x) / 2.0;
        if diff.length_squared() > total_radius * total_radius {
            continue;
        }

        // Assume densities are the same: mass is proportional to size.
        let m1 = t1.scale.x;
        let m2 = t2.scale.x;

        // Solve velocity
        let normal = diff.normalize_or_zero();
        let (v1, w1) = vec_along(p1.velocity, normal);
        let (v2, w2) = vec_along(p2.velocity, normal);
        // v1i + v1f = v2i + v2f
        // v1f = v2i + v2f - v1i
        // Conversation of momentum
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

        if std::mem::discriminant(&c1.cell_type) != std::mem::discriminant(&c2.cell_type) {
            // Cells have different types
            c1.target_scale -= CELL_INTERCOLLISION_DAMAGE;
            c2.target_scale -= CELL_INTERCOLLISION_DAMAGE;
        }
    }
}

fn game_over_check(
    scoreboard: Res<Scoreboard>,
    mut top_text_query: Query<&mut Text, With<TopText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if scoreboard.patient_hp <= 0 || scoreboard.player_hp <= 0 {
        let mut top_text = top_text_query.single_mut();
        top_text.sections[0].value = "GAME OVER".to_owned();
        next_state.set(GameState::Ended);
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
    mut query: Query<(Entity, &mut Transform, &mut Cell)>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for (entity, mut transform, mut cell) in &mut query {
        let scale_diff = cell.target_scale - transform.scale.x;
        let scale_speed = TIME_STEP * 500.0;
        if scale_diff.abs() > scale_speed {
            transform.scale.x += scale_diff.signum() * scale_speed;
        } else {
            transform.scale.x = cell.target_scale;
        }
        transform.scale.y = transform.scale.x;
        let radius = transform.scale.x / 2.0;
        if radius < 15.0 {
            cell.target_scale = 0.0;
        }
        if radius < 5.0 {
            commands.entity(entity).despawn();
            match cell.cell_type {
                CellType::Body { player_hp } => {
                    // Doesn't give player hp when shot
                    scoreboard.patient_hp -= player_hp;
                }
                CellType::Germ { damage: _ } => {
                    scoreboard.score += 1;
                }
            }
        } else if transform.translation.y + radius < boundaries.bottom {
            commands.entity(entity).despawn();
            scoreboard.patient_hp += cell.patient_hp;
            scoreboard.patient_hp = scoreboard.patient_hp.min(100);
        }
    }
}

fn player_bullet_despawner(
    mut commands: Commands,
    boundaries: Res<Boundaries>,
    query: Query<(Entity, &Transform, &PlayerBullet)>,
) {
    for (entity, transform, _) in &query {
        // Allow some buffer space (cells can momentarily go outside screen)
        if transform.translation.y > boundaries.top + 360.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    boundaries: Res<Boundaries>,
    mut spawner: ResMut<Spawner>,
    cell_query: Query<&Cell>,
) {
    if !(spawner.timer.tick(time.delta()).just_finished() && cell_query.is_empty()) {
        return;
    }
    let mut rng = rand::thread_rng();
    let range_x = boundaries.right_wall - boundaries.left_wall;
    let count = rng.gen_range(2..=4);
    for i in 0..count {
        let radius = 45.0;
        let scale = radius * 2.0;
        let min_x = boundaries.left_wall + radius;
        let range_x = range_x - scale;
        let (cell, velocity, material) = if rng.gen_bool(0.5) {
            (
                Cell {
                    cell_type: CellType::Body { player_hp: 10 },
                    target_scale: scale,
                    patient_hp: 1,
                },
                vec2(0.0, -100.0 - rng.gen_range(0.0..100.0)),
                spawner.body_color.clone(),
            )
        } else {
            (
                Cell {
                    cell_type: CellType::Germ { damage: 5 },
                    target_scale: scale,
                    patient_hp: -5,
                },
                vec2(
                    rng.gen_range(-50.0..50.0),
                    -000.0 - rng.gen_range(0.0..100.0),
                ),
                spawner.germ_color.clone(),
            )
        };
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: spawner.circle_mesh.clone(),
                material,
                transform: Transform::from_translation(Vec3::new(
                    rng.gen_range(0.0..range_x) + min_x,
                    boundaries.top + radius + scale * i as f32,
                    1.0,
                ))
                .with_scale(Vec3::new(scale, scale, scale)),
                ..default()
            },
            Physics {
                velocity,
                acceleration: vec2(0.0, -25.0),
                elasticity: 0.9,
            },
            cell,
        ));
    }
}
