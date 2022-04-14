
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};


pub fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::new(400., 400., 0.)),
            material: materials.add(ColorMaterial::from(Color::GRAY)),
            ..MaterialMesh2dBundle::default()
        }
    );
}

