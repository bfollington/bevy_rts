use bevy::{
    prelude::*,
    render::{mesh::*, render_asset::RenderAssetUsages},
};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_tree)
        .run();
}

#[derive(Component)]
struct RotatingTree;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 7.0, 12.0).looking_at(Vec3::new(0., 3., 0.), Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        ..default()
    });

    // Tree
    let tree = generate_tree();
    let (trunk_mesh, leaf_mesh) = create_tree_mesh(&tree);

    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(trunk_mesh),
                material: materials.add(Color::rgb(0.45, 0.3, 0.2)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            RotatingTree,
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(leaf_mesh),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.2, 0.8, 0.2),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                ..default()
            });
        });

    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d::new(
            Vec3::new(0., 1., 0.),
            Vec2::new(10., 10.),
        ))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });
}

fn rotate_tree(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingTree>>) {
    for mut transform in &mut query {
        transform.rotate_y(0.1 * time.delta_seconds());
    }
}

#[derive(Clone)]
struct TreeNode {
    position: Vec3,
    direction: Vec3,
    radius: f32,
    children: Vec<TreeNode>,
    is_leaf: bool,
}

fn generate_tree() -> TreeNode {
    let mut rng = rand::thread_rng();
    let noise = Perlin::new(rng.gen());

    fn generate_branch(
        position: Vec3,
        direction: Vec3,
        radius: f32,
        depth: i32,
        rng: &mut ThreadRng,
        noise: &Perlin,
    ) -> TreeNode {
        let mut node = TreeNode {
            position,
            direction,
            radius,
            children: Vec::new(),
            is_leaf: false,
        };

        if depth == 0 || radius < 0.01 {
            node.is_leaf = true;
            return node;
        }

        let num_branches = rng.gen_range(1..=3);
        for _ in 0..num_branches {
            let noise_input = position * 0.1;
            let noise_value = noise.get([
                noise_input.x as f64,
                noise_input.y as f64,
                noise_input.z as f64,
            ]) as f32;

            let angle = rng.gen_range(-std::f32::consts::PI / 4.0..std::f32::consts::PI / 4.0);
            let length = rng.gen_range(0.5..1.0) * (depth as f32 * 0.2 + 0.8);

            let rotation = Quat::from_rotation_y(noise_value * std::f32::consts::PI * 2.0)
                * Quat::from_rotation_z(angle);

            let new_direction = rotation * direction;
            let new_position = position + new_direction * length;
            let new_radius = radius * rng.gen_range(0.6..0.8);

            let child = generate_branch(
                new_position,
                new_direction,
                new_radius,
                depth - 1,
                rng,
                noise,
            );
            node.children.push(child);
        }

        node
    }

    generate_branch(
        Vec3::ZERO,
        Vec3::Y,
        0.2,
        7, // Adjust this value to control the overall complexity of the tree
        &mut rng,
        &noise,
    )
}

fn create_tree_mesh(tree: &TreeNode) -> (Mesh, Mesh) {
    let mut trunk_positions = Vec::new();
    let mut trunk_normals = Vec::new();
    let mut trunk_indices = Vec::new();
    let mut leaf_positions = Vec::new();
    let mut leaf_normals = Vec::new();
    let mut leaf_uvs = Vec::new();
    let mut leaf_indices = Vec::new();

    fn process_node(
        node: &TreeNode,
        trunk_positions: &mut Vec<[f32; 3]>,
        trunk_normals: &mut Vec<[f32; 3]>,
        trunk_indices: &mut Vec<u32>,
        leaf_positions: &mut Vec<[f32; 3]>,
        leaf_normals: &mut Vec<[f32; 3]>,
        leaf_uvs: &mut Vec<[f32; 2]>,
        leaf_indices: &mut Vec<u32>,
    ) {
        if node.is_leaf {
            add_leaf(node, leaf_positions, leaf_normals, leaf_uvs, leaf_indices);
        } else {
            for child in &node.children {
                add_branch(node, child, trunk_positions, trunk_normals, trunk_indices);
                process_node(
                    child,
                    trunk_positions,
                    trunk_normals,
                    trunk_indices,
                    leaf_positions,
                    leaf_normals,
                    leaf_uvs,
                    leaf_indices,
                );
            }
        }
    }

    process_node(
        tree,
        &mut trunk_positions,
        &mut trunk_normals,
        &mut trunk_indices,
        &mut leaf_positions,
        &mut leaf_normals,
        &mut leaf_uvs,
        &mut leaf_indices,
    );

    let mut trunk_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    trunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, trunk_positions);
    trunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, trunk_normals);
    trunk_mesh.insert_indices(Indices::U32(trunk_indices));

    let mut leaf_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    leaf_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, leaf_positions);
    leaf_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, leaf_normals);
    leaf_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, leaf_uvs);
    leaf_mesh.insert_indices(Indices::U32(leaf_indices));

    (trunk_mesh, leaf_mesh)
}

fn add_branch(
    parent: &TreeNode,
    child: &TreeNode,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
) {
    let segments = 8;
    let base_index = positions.len() as u32;

    let start = parent.position;
    let end = child.position;
    let direction = (end - start).normalize();

    let up = if direction.y.abs() < 0.99 {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let right = direction.cross(up).normalize();
    let forward = right.cross(direction).normalize();

    for i in 0..=segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let cos = angle.cos();
        let sin = angle.sin();

        let offset = right * cos + forward * sin;
        positions.push((start + offset * parent.radius).into());
        positions.push((end + offset * child.radius).into());

        normals.push(offset.into());
        normals.push(offset.into());

        if i < segments {
            let i0 = base_index + i * 2;
            let i1 = base_index + i * 2 + 1;
            let i2 = base_index + (i + 1) * 2;
            let i3 = base_index + (i + 1) * 2 + 1;

            indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
        }
    }
}

fn add_leaf(
    node: &TreeNode,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let leaf_size = 0.2;
    let base_index = positions.len() as u32;

    let direction = node.direction;
    let up = if direction.y.abs() < 0.99 {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let right = direction.cross(up).normalize();
    let forward = right.cross(direction).normalize();

    let corner1 = node.position + (forward + right) * leaf_size;
    let corner2 = node.position + (forward - right) * leaf_size;
    let corner3 = node.position + (-forward - right) * leaf_size;
    let corner4 = node.position + (-forward + right) * leaf_size;

    positions.extend_from_slice(&[
        corner1.into(),
        corner2.into(),
        corner3.into(),
        corner4.into(),
    ]);

    let normal = direction;
    normals.extend_from_slice(&[normal.into(); 4]);

    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

    indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}
