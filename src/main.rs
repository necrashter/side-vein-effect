use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 144.0;

const B0_COLOR: Color = Color::rgb(61.0 / 255.0, 23.0 / 255.0, 102.0 / 255.0);
const F0_COLOR: Color = Color::rgb(111.0 / 255.0, 26.0 / 255.0, 182.0 / 255.0);
const F1_COLOR: Color = Color::rgb(1.0, 0.0, 50.0 / 255.0);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard::default())
        .insert_resource(Boundaries::default())
        .insert_resource(ClearColor(B0_COLOR))
        .add_startup_system(setup)
        .add_systems(
            (
                wall_system,
                physics_objects,
                cell_despawner,
                player_collisions,
                player_movement,
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(update_scoreboard)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Cell;

/// Tracks score, player health, etc.
#[derive(Resource)]
struct Scoreboard {
    score: usize,
    player_hp: usize,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            score: 0,
            player_hp: 100,
        }
    }
}

#[derive(Component)]
enum ScoreboardText {
    Score,
    PlayerHp,
}

#[derive(Component)]
struct Physics {
    velocity: Vec2,
    acceleration: Vec2,
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

    for i in 0..20 {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(F0_COLOR)),
                transform: Transform::from_translation(Vec3::new(
                    i as f32 * 90.0 - 640.0,
                    300.0 + 45.0 * i as f32,
                    1.0,
                ))
                .with_scale(Vec3::new(90.0, 90.0, 0.0)),
                ..default()
            },
            Physics {
                velocity: vec2(i as f32 * 50.0, -100.0),
                acceleration: vec2(0.0, 0.0),
            },
            Cell,
        ));
    }

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
        },
        Player,
    ));

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
                TextStyle {
                    font: asset_server.load("fonts/Kanit-Regular.ttf"),
                    font_size: 32.0,
                    color: TEXT_COLOR,
                },
            ));
            builder.spawn((
                TextBundle::from_section(
                    "100",
                    TextStyle {
                        font: asset_server.load("fonts/Kanit-Regular.ttf"),
                        font_size: 64.0,
                        color: TEXT_COLOR,
                    },
                ),
                ScoreboardText::PlayerHp,
            ));
            builder.spawn(TextBundle::from_section(
                "Score",
                TextStyle {
                    font: asset_server.load("fonts/Kanit-Regular.ttf"),
                    font_size: 32.0,
                    color: TEXT_COLOR,
                },
            ));
            builder.spawn((
                TextBundle::from_section(
                    "0",
                    TextStyle {
                        font: asset_server.load("fonts/Kanit-Regular.ttf"),
                        font_size: 64.0,
                        color: TEXT_COLOR,
                    },
                ),
                ScoreboardText::Score,
            ));
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
        }
    }
}

/// Player-Cell collisions.
fn player_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut player_query: Query<&Transform, With<Player>>,
    cell_query: Query<(Entity, &Transform), With<Cell>>,
    // mut collision_events: EventWriter<CollisionEvent>,
) {
    let player_transform = player_query.single_mut();
    let player_size = player_transform.scale.y;

    // check collision with walls
    for (collider_entity, transform) in &cell_query {
        let dp = transform.translation - player_transform.translation;
        let dist = (dp.x * dp.x) + (dp.y * dp.y);
        let total_radius = (player_size + transform.scale.y) / 2.0;
        let rad2 = total_radius * total_radius;
        if dist <= rad2 {
            // Sends a collision event so that other systems can react to the collision
            // collision_events.send_default();

            scoreboard.score += 1;
            scoreboard.player_hp -= 1;
            commands.entity(collider_entity).despawn();
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
            physics.velocity.x *= -1.0;
        } else if transform.translation.x + radius > boundaries.right_wall {
            transform.translation.x = boundaries.right_wall - radius;
            physics.velocity.x *= -1.0;
        }
    }
}

fn cell_despawner(
    mut commands: Commands,
    boundaries: Res<Boundaries>,
    query: Query<(Entity, &Transform), With<Cell>>,
) {
    for (entity, transform) in &query {
        let radius = transform.scale.x / 2.0;
        if transform.translation.y + radius < boundaries.bottom {
            commands.entity(entity).despawn();
        }
    }
}
