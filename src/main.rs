use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 144.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(Color::rgb(0.2, 0.1, 0.1)))
        .add_startup_system(setup)
        .add_systems((player_movement,).in_schedule(CoreSchedule::FixedUpdate))
        .add_system(update_scoreboard)
        // Configure how frequently our gameplay systems are run
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .run();
}

#[derive(Component)]
struct Player;

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.5))),
            transform: Transform::from_translation(Vec3::new(0.0, -50.0, 1.0))
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
                color: SCORE_COLOR,
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

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
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

    transform.translation.x += dx * 500.0 * TIME_STEP;
    transform.translation.y += dy * 500.0 * TIME_STEP;
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}
