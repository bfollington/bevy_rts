use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use rand::prelude::*;

// Particle component
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: Timer,
    initial_color: Color,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create a basic circle mesh for particles
    let mesh = meshes.add(Circle::new(5.));

    // Store the mesh and colors as resources for easy access
    commands.insert_resource(ParticleMesh(mesh));
    commands.insert_resource(ParticleColors {
        blue: Color::rgb(0.0, 0.0, 1.0),
        orange: Color::rgb(1.0, 0.5, 0.0),
    });
}

fn spawn_particles(
    mut commands: Commands,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    particle_mesh: Res<ParticleMesh>,
    particle_colors: Res<ParticleColors>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            let spawn_count = 50;
            let mut rng = thread_rng();

            for _ in 0..spawn_count {
                let velocity =
                    Vec2::new(rng.gen_range(-100.0..100.0), rng.gen_range(-100.0..100.0));
                let lifetime = Timer::from_seconds(rng.gen_range(0.5..1.5), TimerMode::Once);
                let initial_color = if rng.gen_bool(0.5) {
                    particle_colors.blue
                } else {
                    particle_colors.orange
                };

                let material = materials.add(ColorMaterial::from(initial_color));

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: particle_mesh.0.clone().into(),
                        material,
                        transform: Transform::from_translation(Vec3::new(
                            position.x - window.width() / 2.0,
                            position.y - window.height() / 2.0,
                            0.0,
                        )),
                        ..default()
                    },
                    Particle {
                        velocity,
                        lifetime,
                        initial_color,
                    },
                ));
            }
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(
        Entity,
        &mut Transform,
        &mut Particle,
        &Handle<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transform, mut particle, material_handle) in particle_query.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity.extend(0.) * time.delta_seconds();

            // Apply some "gravity" effect
            particle.velocity.y -= 50. * time.delta_seconds();

            // Update alpha based on remaining lifetime
            let alpha = particle.lifetime.fraction_remaining();
            if let Some(material) = materials.get_mut(material_handle) {
                material.color = particle.initial_color.with_alpha(alpha);
            }
        }
    }
}

// Resources
#[derive(Resource)]
struct ParticleMesh(Handle<Mesh>);

#[derive(Resource)]
struct ParticleColors {
    blue: Color,
    orange: Color,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_particles, update_particles))
        .run();
}
