use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{WindowPlugin, Window, PrimaryWindow};

struct Player;

impl Component for Player {
    type Storage = bevy::ecs::component::SparseStorage;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy FPS".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, (setup, hide_cursor))
        .add_systems(Update, (player_movement, mouse_look))
        .run();
}

// Hides the cursor
fn hide_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor.visible = false;
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>, mut meshes: ResMut<Assets<Mesh>>) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0, subdivisions: 0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // Player
    commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Player,
    ));
}

fn player_movement(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keys.pressed(KeyCode::W) {
            direction.z -= 1.0;
        }
        if keys.pressed(KeyCode::S) {
            direction.z += 1.0;
        }
        if keys.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }
        if keys.pressed(KeyCode::D) {
            direction.x += 1.0;
        }

        let translation = &mut transform.translation;
        *translation += time.delta_seconds() * direction * 5.0;
    }
}

fn mouse_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut rotation = Vec2::ZERO;

    for event in mouse_motion_events.iter() {
        rotation += event.delta;
    }

    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_axis_angle(Vec3::Y, rotation.x * 0.001)
            * transform.rotation;
        transform.rotation = Quat::from_axis_angle(Vec3::X, -rotation.y * 0.001)
            * transform.rotation;
    }
}
