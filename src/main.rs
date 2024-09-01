use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls, RtsCameraPlugin, RtsCameraSystemSet};
use std::f32::consts::TAU;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_unit,
                lock_or_jump,
                toggle_controls,
                move_selected_unit,
                update_selection_visual,
                smooth_unit_movement,
            )
                .chain()
                .before(RtsCameraSystemSet),
        )
        .run();
}

#[derive(Component)]
struct Move;

#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct TargetPosition(Vec3);

#[derive(Resource)]
struct UnitMaterials {
    normal: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(80.0, 80.0)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        Ground,
    ));

    // Some "terrain"
    let terrain_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(15.0, 1.0, 5.0)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(15.0, 0.5, -5.0),
            ..default()
        },
        Ground,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(10.0, 5.0, 15.0)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(-15.0, 2.5, 0.0),
            ..default()
        },
        Ground,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(12.5)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, -23.0),
            ..default()
        },
        Ground,
    ));

    // Create and store unit materials
    let unit_materials = UnitMaterials {
        normal: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.2),
            ..default()
        }),
        selected: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.8, 0.2),
            ..default()
        }),
    };

    // Spawn selectable units
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Capsule3d::new(0.25, 1.25)),
                    material: unit_materials.normal.clone(),
                    transform: Transform::from_xyz(x as f32 * 0.7, 0.75, z as f32 * 0.7),
                    ..default()
                },
                Selectable,
                PickableBundle::default(),
                On::<Pointer<Click>>::run(select_unit),
            ));
        }
    }

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            150.0f32.to_radians(),
            -40.0f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Help text
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "\
Press K to jump to the moving unit
Hold L to lock onto the moving unit
Press T to toggle controls (K and L will still work)
Left-click to select units
Right-click to move selected units"
                    .to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        ..default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle::default(),
        RtsCamera {
            height_max: 50.0,
            min_angle: 35.0f32.to_radians(),
            smoothness: 0.1,
            target_focus: Transform::from_xyz(3.0, 0.0, -3.0),
            target_zoom: 0.2,
            ..default()
        },
        RtsCameraControls {
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            button_rotate: MouseButton::Middle, // Changed from Right to Middle
            lock_on_rotate: true,
            button_drag: Some(MouseButton::Middle),
            lock_on_drag: true,
            edge_pan_width: 0.1,
            pan_speed: 25.0,
            ..default()
        },
    ));

    // Add UnitMaterials as a resource
    commands.insert_resource(unit_materials);
}

// Move a unit in a circle
fn move_unit(
    time: Res<Time>,
    mut cube_q: Query<&mut Transform, With<Move>>,
    mut angle: Local<f32>,
) {
    if let Ok(mut cube_tfm) = cube_q.get_single_mut() {
        // Rotate 20 degrees a second, wrapping around to 0 after a full rotation
        *angle += 20f32.to_radians() * time.delta_seconds() % TAU;
        // Convert angle to position
        let pos = Vec3::new(angle.sin() * 7.5, 0.75, angle.cos() * 7.5);
        cube_tfm.translation = pos;
    }
}

// Either jump to the moving unit (press K) or lock onto it (hold L)
fn lock_or_jump(
    key_input: Res<ButtonInput<KeyCode>>,
    cube_q: Query<&Transform, With<Move>>,
    mut cam_q: Query<&mut RtsCamera>,
) {
    for cube in cube_q.iter() {
        for mut cam in cam_q.iter_mut() {
            if key_input.pressed(KeyCode::KeyL) {
                cam.target_focus.translation = cube.translation;
                cam.snap = true;
            }
            if key_input.just_pressed(KeyCode::KeyK) {
                cam.target_focus.translation = cube.translation;
                cam.target_zoom = 0.4;
            }
        }
    }
}

fn toggle_controls(
    mut controls_q: Query<&mut RtsCameraControls>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    for mut controls in controls_q.iter_mut() {
        if key_input.just_pressed(KeyCode::KeyT) {
            controls.enabled = !controls.enabled;
        }
    }
}

fn select_unit(event: Listener<Pointer<Click>>, mut commands: Commands, query: Query<&Selected>) {
    let entity = event.target;
    if query.get(entity).is_ok() {
        commands.entity(entity).remove::<Selected>();
    } else {
        commands.entity(entity).insert(Selected);
    }
}

fn update_selection_visual(
    unit_materials: Res<UnitMaterials>,
    mut query: Query<(&mut Handle<StandardMaterial>, Option<&Selected>), With<Selectable>>,
) {
    for (mut material, selected) in query.iter_mut() {
        *material = if selected.is_some() {
            unit_materials.selected.clone()
        } else {
            unit_materials.normal.clone()
        };
    }
}

fn move_selected_unit(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected_units: Query<Entity, With<Selected>>,
    ground: Query<&GlobalTransform, With<Ground>>,
    mut commands: Commands,
) {
    if !mouse_button_input.just_pressed(MouseButton::Right) {
        return;
    }

    let (camera, camera_transform) = camera_q.single();
    let window = windows.single();

    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let ray = match camera.viewport_to_world(camera_transform, cursor_position) {
        Some(r) => r,
        None => return,
    };

    let ground_transform = match ground.iter().next() {
        Some(gt) => gt,
        None => return,
    };

    let ground_plane = InfinitePlane3d::new(Vec3::Y);
    let distance = match ray.intersect_plane(ground_transform.translation(), ground_plane) {
        Some(d) => d,
        None => return,
    };

    let world_position = ray.get_point(distance);
    for entity in selected_units.iter() {
        commands
            .entity(entity)
            .insert(TargetPosition(world_position));
    }
}

fn smooth_unit_movement(
    time: Res<Time>,
    mut units: Query<(Entity, &mut Transform, &TargetPosition)>,
    mut commands: Commands,
) {
    for (entity, mut transform, target) in units.iter_mut() {
        let direction = target.0 - transform.translation;
        if direction.length() > 0.1 {
            let movement = direction.normalize() * 5.0 * time.delta_seconds();
            transform.translation += movement;
            // Ensure the unit stays on the ground
            transform.translation.y = 0.75; // Assuming 0.75 is the ground level for units
        } else {
            // Remove the TargetPosition component when the unit reaches its destination
            commands.entity(entity).remove::<TargetPosition>();
        }
    }
}
