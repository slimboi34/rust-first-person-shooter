use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, WindowPlugin, Window, PrimaryWindow};

struct Player;

impl Component for Player {
    type Storage = bevy::ecs::component::SparseStorage;
}

#[derive(Resource)]
struct MouseSettings {
    sensitivity: f32,
    smoothing: f32,
}

#[derive(Resource)]
struct MovementSettings {
    base_speed: f32,
    sprint_multiplier: f32,
}

#[derive(Resource)]
struct CameraState {
    yaw: f32,
    pitch: f32,
    target_yaw: f32,
    target_pitch: f32,
}

#[derive(Resource)]
struct WeaponSettings {
    fire_rate: f32,
}

#[derive(Component)]
struct Projectile {
    velocity: Vec3,
}

#[derive(Component)]
struct Collider {
    half_extents: Vec3,
}

#[derive(Component)]
struct Explosion;

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
        .insert_resource(MouseSettings {
            sensitivity: 0.015,
            smoothing: 0.1,
        })
        .insert_resource(MovementSettings {
            base_speed: 15.0,
            sprint_multiplier: 1.5,
        })
        .insert_resource(CameraState {
            yaw: 0.0,
            pitch: 0.0,
            target_yaw: 0.0,
            target_pitch: 0.0,
        })
        .insert_resource(WeaponSettings {
            fire_rate: 10.0, // 10 shots per second
        })
        .add_systems(Startup, (setup, hide_cursor))
        .add_systems(Update, (player_movement, mouse_look, fire_weapon, move_projectiles, check_collisions))
        .run();
}

// Hides the cursor and locks it to the window
fn hide_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Confined;
    }
}

// Linear interpolation function for f32
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
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

    // Ground, walls, and other setup code omitted for brevity...

    // Rifle (a simple cuboid as a placeholder for the actual model)
    let rifle_material = materials.add(Color::rgb(0.3, 0.3, 0.3).into());
    let rifle_mesh = meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.5))); // Create a cuboid to represent the rifle

    // Position the rifle in front of the camera, centered horizontally, and rotated to face forward
    commands.spawn((
        PbrBundle {
            mesh: rifle_mesh,
            material: rifle_material,
            transform: Transform {
                translation: Vec3::new(0.0, -0.2, -0.6), // Centered on the X-axis, slight downwards Y, forward Z
                rotation: Quat::from_rotation_y(0.0), // No additional rotation needed if it's already facing forward
                ..Default::default()
            },
            ..Default::default()
        }
    ))
    .set_parent(camera_entity); // Attach the rifle to the camera
}


    // Ground with Grid Lines
    let ground_size = 20.0;  // Increased the ground size
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: ground_size, subdivisions: 20 })),
            material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()), // Dark grey ground
            ..Default::default()
        },
        Collider {
            half_extents: Vec3::new(ground_size / 2.0, 0.1, ground_size / 2.0),
        },
    ));

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
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(ground_size + wall_thickness, wall_height, wall_thickness))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
            transform: Transform::from_xyz(0.0, wall_height / 2.0, -half_size - wall_thickness / 2.0),
            ..Default::default()
        },
        Collider {
            half_extents: Vec3::new((ground_size + wall_thickness) / 2.0, wall_height / 2.0, wall_thickness / 2.0),
        },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(ground_size + wall_thickness, wall_height, wall_thickness))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
            transform: Transform::from_xyz(0.0, wall_height / 2.0, half_size + wall_thickness / 2.0),
            ..Default::default()
        },
        Collider {
            half_extents: Vec3::new((ground_size + wall_thickness) / 2.0, wall_height / 2.0, wall_thickness / 2.0),
        },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(wall_thickness, wall_height, ground_size + wall_thickness))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
            transform: Transform::from_xyz(-half_size - wall_thickness / 2.0, wall_height / 2.0, 0.0),
            ..Default::default()
        },
        Collider {
            half_extents: Vec3::new(wall_thickness / 2.0, wall_height / 2.0, (ground_size + wall_thickness) / 2.0),
        },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(wall_thickness, wall_height, ground_size + wall_thickness))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()), // Grey walls
            transform: Transform::from_xyz(half_size + wall_thickness / 2.0, wall_height / 2.0, 0.0),
            ..Default::default()
        },
        Collider {
            half_extents: Vec3::new(wall_thickness / 2.0, wall_height / 2.0, (ground_size + wall_thickness) / 2.0),
        },
    ));

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
    movement_settings: Res<MovementSettings>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let height = 1.7; // Constant height
    let ground_size = 20.0; // Size of the ground

    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Manually calculate forward and right vectors based on rotation
        let forward = transform.rotation * Vec3::Z;
        let right = transform.rotation * Vec3::X;

        // Determine movement direction
        if keys.pressed(KeyCode::W) {
            direction -= forward;
        }
        if keys.pressed(KeyCode::S) {
            direction += forward;
        }
        if keys.pressed(KeyCode::A) {
            direction -= right;
        }
        if keys.pressed(KeyCode::D) {
            direction += right;
        }

        // Normalize direction and apply movement
        if direction.length() > 0.0 {
            direction = direction.normalize();

            // Check for sprinting
            let speed = if keys.pressed(KeyCode::ShiftLeft) {
                movement_settings.base_speed * movement_settings.sprint_multiplier
            } else {
                movement_settings.base_speed
            };

            let new_translation = transform.translation + time.delta_seconds() * direction * speed;

            // Collision detection: Ensure player doesn't move through the walls
            let half_size = ground_size / 2.0 - 0.5; // Adjust for some buffer space
            if new_translation.x > -half_size && new_translation.x < half_size &&
               new_translation.z > -half_size && new_translation.z < half_size {
                transform.translation = new_translation;
            }

            // Maintain constant height
            transform.translation.y = height;
        }
    }
}

fn mouse_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
    mouse_settings: Res<MouseSettings>,
    mut camera_state: ResMut<CameraState>,
) {
    let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01; // Limit pitch to prevent flipping

    for event in mouse_motion_events.iter() {
        camera_state.target_yaw += event.delta.x * mouse_settings.sensitivity;
        camera_state.target_pitch -= event.delta.y * mouse_settings.sensitivity;

        // Clamp the pitch to avoid flipping
        camera_state.target_pitch = camera_state.target_pitch.clamp(-pitch_limit, pitch_limit);
    }

    // Smoothly interpolate to the target yaw and pitch
    camera_state.yaw = lerp(camera_state.yaw, camera_state.target_yaw, mouse_settings.smoothing);
    camera_state.pitch = lerp(camera_state.pitch, camera_state.target_pitch, mouse_settings.smoothing);

    // Update the rotation based on the interpolated yaw and pitch
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(camera_state.yaw) * Quat::from_rotation_x(camera_state.pitch);
    }
}

fn fire_weapon(
    mouse_button_input: Res<Input<MouseButton>>,
    time: Res<Time>,
    weapon_settings: Res<WeaponSettings>,
    mut last_shot_time: Local<f32>,
    mut commands: Commands,
    query: Query<&Transform, With<Player>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) && (*last_shot_time + 1.0 / weapon_settings.fire_rate) <= time.elapsed_seconds() {
        *last_shot_time = time.elapsed_seconds();

        if let Ok(player_transform) = query.get_single() {
            let spawn_transform = Transform {
                translation: player_transform.translation + player_transform.rotation * Vec3::new(0.0, 0.0, -1.0),
                rotation: player_transform.rotation,
                ..Default::default()
            };

            // Spawn a projectile using UVSphere instead of Icosphere
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.05, sectors: 16, stacks: 16 })),
                    material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()), // Red projectile
                    transform: spawn_transform,
                    ..Default::default()
                },
                Projectile {
                    velocity: player_transform.rotation * Vec3::new(0.0, 0.0, -30.0), // Set the speed of the projectile
                },
            ));
        }
    }
}
fn move_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(Entity, &mut Transform, &Projectile)>,
        Query<(&Transform, &Collider)>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut despawn_entities = vec![];
    let mut explosions = vec![];

    // Collect projectile data
    let projectiles: Vec<_> = param_set.p0().iter_mut().map(|(entity, mut transform, projectile)| {
        let new_translation = transform.translation + projectile.velocity * time.delta_seconds();
        transform.translation = new_translation;

        if transform.translation.length() > 50.0 {
            despawn_entities.push(entity);
        }

        (entity, transform.translation)
    }).collect();

    // Collect collider data
    let colliders: Vec<_> = param_set.p1().iter().map(|(transform, collider)| {
        (transform.translation, collider.half_extents)
    }).collect();

    // Check for collisions
    for (entity, translation) in projectiles {
        for (collider_translation, collider_half_extents) in &colliders {
            if check_aabb_collision(
                translation,
                Vec3::splat(0.05), // Projectile radius as half extents
                *collider_translation,
                *collider_half_extents,
            ) {
                despawn_entities.push(entity);

                // Create a new explosion effect
                explosions.push((translation, Transform::from_translation(translation)));
                break; // Stop checking other colliders
            }
        }
    }

    for (explosion_translation, transform) in explosions {
        // Create a new explosion mesh and material
        let explosion_mesh = meshes.add(Mesh::from(shape::UVSphere { radius: 0.3, sectors: 16, stacks: 16 }));
        let explosion_material = materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 0.5, 0.0),
            ..Default::default()
        });

        // Spawn the explosion effect
        commands.spawn(PbrBundle {
            mesh: explosion_mesh,
            material: explosion_material,
            transform,
            ..Default::default()
        });
    }

    for entity in despawn_entities {
        commands.entity(entity).despawn();
    }
}

// Dummy function for collision detection (replace with actual collision detection logic)
fn check_aabb_collision(
    pos1: Vec3,
    half_extents1: Vec3,
    pos2: Vec3,
    half_extents2: Vec3,
) -> bool {
    // AABB collision detection logic
    let delta = pos1 - pos2;
    let intersect_x = delta.x.abs() <= half_extents1.x + half_extents2.x;
    let intersect_y = delta.y.abs() <= half_extents1.y + half_extents2.y;
    let intersect_z = delta.z.abs() <= half_extents1.z + half_extents2.z;
    intersect_x && intersect_y && intersect_z
}



fn check_collisions(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    collider_query: Query<(Entity, &Transform), Without<Projectile>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (projectile_entity, projectile_transform) in projectile_query.iter() {
        for (_collider_entity, collider_transform) in collider_query.iter() {
            let distance = projectile_transform.translation.distance(collider_transform.translation);
            if distance < 0.1 { // Collision threshold
                // Despawn the projectile
                commands.entity(projectile_entity).despawn();

                // Spawn an explosion
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.3, sectors: 16, stacks: 16 })),
                        material: materials.add(Color::rgb(1.0, 0.5, 0.0).into()), // Orange explosion
                        transform: *projectile_transform,
                        ..Default::default()
                    },
                    Explosion,
                ));

                // You can add more effects like a sound or particles here if desired
            }
        }
    }
}
