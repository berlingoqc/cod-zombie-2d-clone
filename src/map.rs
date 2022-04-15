
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};


#[derive(Component)]
pub struct Collider {}

#[derive(Component)]
pub struct CollisionEvent {}

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
    info: MapElementPosition
}

#[derive(Component)]
pub struct MapElementPosition {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: i32
}

impl WallBundle {
    fn new(info: MapElementPosition) -> WallBundle {
        WallBundle{
            sprite_bundle: SpriteBundle{
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.25, 0.25),
                    custom_size: Some(info.size),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: info.position.extend(10.0),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            },
           collider: Collider{},
           info
        }
    }
}

pub fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let floor = Vec3::new(400., 400., 0.);
    let walls = vec![
        // RIGHT
        MapElementPosition{
            position: Vec2::new(200., 0.),
            size: Vec2::new(10., 400.),
            rotation: 0,
        },
        // LEFT
        MapElementPosition{
            position: Vec2::new(-200., 0.),
            size: Vec2::new(10., 400.),
            rotation: 0,
        },
        // TOP
        MapElementPosition{
            position: Vec2::new(0., 200.),
            size: Vec2::new(400., 10.),
            rotation: 0,
        },
        // BOTTOM
        MapElementPosition{
            position: Vec2::new(0., -200.),
            size: Vec2::new(400., 10.),
            rotation: 0,
        }
    ];

    commands.spawn_bundle(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::new(400., 400., 0.)),
            material: materials.add(ColorMaterial::from(Color::GRAY)),
            ..MaterialMesh2dBundle::default()
        }
    );


    walls.into_iter().for_each(|w| {
        commands.spawn_bundle(WallBundle::new(w));
    });

}

