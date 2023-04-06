use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 144.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const B0_COLOR: Color = Color::rgb(61.0 / 255.0, 23.0 / 255.0, 102.0 / 255.0);
const F0_COLOR: Color = Color::rgb(111.0 / 255.0, 26.0 / 255.0, 182.0 / 255.0);
const F1_COLOR: Color = Color::rgb(1.0, 0.0, 50.0 / 255.0);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(Boundaries::default())
        .insert_resource(ClearColor(B0_COLOR))
        .add_startup_system(setup)
        .add_systems(
            (
                wall_system,
                cell_collisions,
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

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: usize,
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
                    60.0,
                    1.0,
                ))
                .with_scale(Vec3::new(90.0, 90.0, 0.0)),
                ..default()
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
        Player,
    ));

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/Kanit-Regular.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/Kanit-Regular.ttf"),
                font_size: SCOREBOARD_FONT_SIZE,
                color: TEXT_COLOR,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        }),
    );
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
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut();
    let mut dx = 0.0;
    let mut dy = 0.0;

    if keyboard_input.pressed(KeyCode::A) {
        dx -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        dx += 1.0;
    }
    if keyboard_input.pressed(KeyCode::W) {
        dy += 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
        dy -= 1.0;
    }

    let radius = transform.scale.x / 2.0;
    let left_bound = boundaries.left_wall + radius;
    let right_bound = boundaries.right_wall - radius;
    let top_bound = boundaries.top - radius;
    let bottom_bound = boundaries.bottom + radius;

    transform.translation.x += dx * 500.0 * TIME_STEP;
    transform.translation.x = transform.translation.x.clamp(left_bound, right_bound);
    transform.translation.y += dy * 500.0 * TIME_STEP;
    transform.translation.y = transform.translation.y.clamp(bottom_bound, top_bound);
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
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
            commands.entity(collider_entity).despawn();
        }
    }
}

/// Cell boundary collisions.
fn cell_collisions(boundaries: Res<Boundaries>, mut cell_query: Query<(&mut Transform, &Cell)>) {
    for (mut transform, _) in &mut cell_query {
        let radius = transform.scale.x / 2.0;
        if transform.translation.x - radius < boundaries.left_wall {
            transform.translation.x = boundaries.left_wall + radius;
        } else if transform.translation.x + radius > boundaries.right_wall {
            transform.translation.x = boundaries.right_wall - radius;
        }
    }
}
