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

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Camera and Player
    let camera_entity = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.7, 5.0).looking_at(Vec3::ZERO, Vec3::Y), // Set height to 1.7 meters
            ..Default::default()
        },
        Player, // Attach the Player component to the camera for simplicity
    )).id();

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // Ground with Grid Lines
    let ground_size = 20.0;  // Increased the ground size
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: ground_size, subdivisions: 20 })),
        material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()), // Dark grey ground
        ..Default::default()
    });

    // Grid Lines
    for i in 0..=20 {
        let offset = i as f32 * (ground_size / 20.0) - (ground_size / 2.0);
        // Vertical line
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.02, 0.02, ground_size))),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()), // White lines
            transform: Transform::from_xyz(offset, 0.01, 0.0),
            ..Default::default()
        });
        // Horizontal line
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(ground_size, 0.02, 0.02))),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()), // White lines
            transform: Transform::from_xyz(0.0, 0.01, offset),
            ..Default::default()
        });
    }

    // Walls
    let wall_thickness = 0.2;
    let wall_height = 3.0;
    let half_size = ground_size / 2.0;

    // Four walls surrounding the ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(ground_size + wall_thickness, wall_height, wall_thickness))),
        material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
        transform: Transform::from_xyz(0.0, wall_height / 2.0, -half_size - wall_thickness / 2.0),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(ground_size + wall_thickness, wall_height, wall_thickness))),
        material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
        transform: Transform::from_xyz(0.0, wall_height / 2.0, half_size + wall_thickness / 2.0),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(wall_thickness, wall_height, ground_size + wall_thickness))),
        material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
        transform: Transform::from_xyz(-half_size - wall_thickness / 2.0, wall_height / 2.0, 0.0),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(wall_thickness, wall_height, ground_size + wall_thickness))),
        material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
        transform: Transform::from_xyz(half_size + wall_thickness / 2.0, wall_height / 2.0, 0.0),
        ..Default::default()
    });

    // Assault Rifle (a simple cuboid as a placeholder for the actual model)
    let rifle_material = materials.add(Color::rgb(0.3, 0.3, 0.3).into());
    let rifle_mesh = meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.5))); // Create a cuboid to represent the rifle

    commands.spawn(PbrBundle {
        mesh: rifle_mesh,
        material: rifle_material,
        transform: Transform {
            translation: Vec3::new(0.3, -0.2, -0.5), // Position relative to the camera
            rotation: Quat::from_rotation_x(std::f32::consts::PI / 2.0), // Rotate to align correctly
            ..Default::default()
        },
        ..Default::default()
    })
    .set_parent(camera_entity); // Attach the rifle to the camera
}

fn player_movement(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let speed = 15.0; // Increased speed for faster movement
    let height = 1.7; // Constant height
    let ground_size = 20.0; // Size of the ground

    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Manually calculate forward and right vectors based on rotation
        let forward = transform.rotation * Vec3::Z;
        let right = transform.rotation * Vec3::X;

        // Forward/Backward movement
        if keys.pressed(KeyCode::W) {
            direction -= forward;
        }
        if keys.pressed(KeyCode::S) {
            direction += forward;
        }

        // Left/Right movement
        if keys.pressed(KeyCode::A) {
            direction -= right;
        }
        if keys.pressed(KeyCode::D) {
            direction += right;
        }

        // Apply the movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
            let new_translation = transform.translation + time.delta_seconds() * direction * speed;

            // Collision detection: Ensure player doesn't move through the walls
            let half_size = ground_size / 2.0 - 0.5; // Adjust for some buffer space
            if new_translation.x > -half_size && new_translation.x < half_size &&
               new_translation.z > -half_size && new_translation.z < half_size {
                transform.translation = new_translation;
            }
            transform.translation.y = height; // Lock the Y position to maintain height
        }
    }
}

fn mouse_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let sensitivity = 0.001;

    for event in mouse_motion_events.iter() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_axis_angle(Vec3::Y, event.delta.x * sensitivity)
                * transform.rotation;
            transform.rotation = Quat::from_axis_angle(Vec3::X, -event.delta.y * sensitivity)
                * transform.rotation;
        }
    }
}
