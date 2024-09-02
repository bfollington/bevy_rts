use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_rts_camera::{RtsCamera, RtsCameraControls, RtsCameraPlugin};

const CUBE_SIZE: f32 = 0.5;
const PILE_SIZE: i32 = 3;
const PILE_HEIGHT: i32 = 2;
const CUBE_SPACING: f32 = 1.1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Physics RTS Demo".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (hover_highlight, drag_object, release_object))
        .run();
}

#[derive(Component)]
struct Pickable;

#[derive(Component)]
struct Dragging {
    offset: Vec3,
    start_position: Vec3,
}

#[derive(Component, Clone)]
struct HighlightedCube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.7, 0.0)),
        ..default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RtsCamera::default(),
        RtsCameraControls {
            edge_pan_width: 0.1,
            zoom_sensitivity: 5.0,
            pan_speed: 10.0,
            ..default()
        },
    ));

    // Ground plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(25.0, 0.1, 25.0),
    ));

    // Cube pile
    let cube_mesh = meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE));
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        metallic: 0.7,
        perceptual_roughness: 0.2,
        ..default()
    });

    for x in -PILE_SIZE..=PILE_SIZE {
        for z in -PILE_SIZE..=PILE_SIZE {
            for y in 0..PILE_HEIGHT {
                let position = Vec3::new(
                    x as f32 * CUBE_SIZE * CUBE_SPACING,
                    y as f32 * CUBE_SIZE * CUBE_SPACING + CUBE_SIZE / 2.0,
                    z as f32 * CUBE_SIZE * CUBE_SPACING,
                );

                commands.spawn((
                    PbrBundle {
                        mesh: cube_mesh.clone(),
                        material: cube_material.clone(),
                        transform: Transform::from_translation(position),
                        ..default()
                    },
                    RigidBody::Dynamic,
                    Collider::cuboid(CUBE_SIZE / 2.0, CUBE_SIZE / 2.0, CUBE_SIZE / 2.0),
                    PickableBundle::default(),
                    Pickable,
                    On::<Pointer<Over>>::target_insert(HighlightedCube),
                    On::<Pointer<Out>>::target_remove::<HighlightedCube>(),
                ));
            }
        }
    }
}

fn hover_highlight(
    mut materials: ResMut<Assets<StandardMaterial>>,
    highlighted_cubes: Query<&Handle<StandardMaterial>, With<HighlightedCube>>,
    unhighlighted_cubes: Query<
        &Handle<StandardMaterial>,
        (With<Pickable>, Without<HighlightedCube>),
    >,
) {
    for material_handle in highlighted_cubes.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = Color::srgb(1.0, 0.5, 0.5);
        }
    }

    for material_handle in unhighlighted_cubes.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = Color::srgb(0.8, 0.7, 0.6);
        }
    }
}

fn drag_object(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut object_query: ParamSet<(
        Query<(Entity, &Transform, &PickSelection), With<Pickable>>,
        Query<(Entity, &mut Transform, &Dragging)>,
    )>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window.single();

    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                let mut pickable_query = object_query.p0();
                for (entity, transform, pick_selection) in pickable_query.iter() {
                    if pick_selection.is_selected {
                        let distance = ray.origin.distance(transform.translation);
                        let world_position = ray.get_point(distance);
                        let offset = transform.translation - world_position;

                        commands.entity(entity).insert(Dragging {
                            offset,
                            start_position: transform.translation,
                        });
                        commands.entity(entity).remove::<RigidBody>();
                        break;
                    }
                }
            }
        }
    }

    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                let mut dragging_query = object_query.p1();
                for (_, mut transform, dragging) in dragging_query.iter_mut() {
                    let distance = ray.origin.distance(transform.translation);
                    let world_position = ray.get_point(distance);
                    transform.translation = world_position + dragging.offset;
                }
            }
        }
    }
}

fn release_object(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    query: Query<(Entity, &Transform, &Dragging)>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for (entity, transform, dragging) in query.iter() {
            let velocity = (transform.translation - dragging.start_position) / time.delta_seconds();
            commands.entity(entity).remove::<Dragging>();
            commands.entity(entity).insert(RigidBody::Dynamic);
            commands.entity(entity).insert(LinearVelocity(velocity));
        }
    }
}
