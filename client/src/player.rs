use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{
    game::Zombie,
    map::data::{MapElementPosition, MovementCollider, ProjectileCollider},
};

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Default, Component)]
pub struct Player {}

#[derive(Default, Component)]
pub struct Projectile {}

#[derive(Default, Component)]
pub struct ExpiringComponent {
    pub created_at: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub v: Vec2,
}

#[derive(Component)]
pub struct MainCamera;

pub fn apply_velocity(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &Velocity, Entity)>,
) {
    for (mut transform, velocity, entity) in query.iter_mut() {
        let x_vel = velocity.v.x * TIME_STEP;
        let y_vel = velocity.v.y * TIME_STEP;
        if x_vel == 0. && y_vel == 0. {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += x_vel;
        transform.translation.y += y_vel;
    }
}

pub fn movement_projectile(
    mut commands: Commands,
    time: Res<Time>,
    projectile_query: Query<(Entity, &Transform, &ExpiringComponent), With<Projectile>>,
    collider_query: Query<
        (Entity, &Transform, &MapElementPosition, Option<&Zombie>),
        (
            With<ProjectileCollider>,
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,
) {
    'outer: for (projectile_entity, transform, expiring) in projectile_query.iter() {
        if expiring.created_at + expiring.duration <= time.time_since_startup().as_secs_f32() {
            commands.entity(projectile_entity).despawn();
            break;
        }
        for (hit_entity, transform_collider, info, zombie) in collider_query.iter() {
            let collision = collide(
                transform.translation,
                Vec2::new(5., 5.),
                transform_collider.translation,
                info.size,
            );
            if collision.is_some() {
                if let Some(_zombie) = zombie {
                    commands.entity(hit_entity).despawn();
                }
                commands.entity(projectile_entity).despawn();
                break 'outer;
            }
        }
    }
}

fn get_cursor_location(
    wnds: &Windows,
    q_camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Vec2 {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    let window = match camera.target {
        bevy::render::camera::RenderTarget::Window(w) => w,
        _ => panic!("camera not rendering to windows"),
    };

    // get the window that the camera is displaying to
    let wnd = wnds.get(window).unwrap();

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        return world_pos;
    } else {
        return Vec2::new(0., 0.);
    }
}

pub fn input_player(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<&mut Transform, With<Player>>,
    collider_query: Query<
        (Entity, &Transform, &MapElementPosition),
        (
            With<MovementCollider>,
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let mut player_transform = query.single_mut();

    let mut movement = Vec3::default();
    let mut moved = false;

    if keyboard_input.pressed(KeyCode::W) {
        movement += Vec3::new(0., 1., 0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::S) {
        movement += Vec3::new(0., -1., 0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::A) {
        movement += Vec3::new(-1., 0., 0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::D) {
        movement += Vec3::new(1., 0., 0.);
        moved = true;
    }

    if buttons.just_pressed(MouseButton::Left) || keyboard_input.pressed(KeyCode::Space) {
        let mouse_location = get_cursor_location(&wnds, &q_camera).normalize_or_zero();

        commands
            .spawn()
            .insert(Projectile {})
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: player_transform.translation,
                    ..Transform::default()
                },
                sprite: Sprite {
                    color: Color::BISQUE,
                    custom_size: Some(Vec2::new(5.0, 5.0)),
                    ..Sprite::default()
                },
                ..SpriteBundle::default()
            })
            .insert(ExpiringComponent {
                created_at: time.time_since_startup().as_secs_f32(),
                duration: 2.,
            })
            .insert(ProjectileCollider {})
            .insert(Velocity {
                v: mouse_location * 1000.,
            });
    }

    if !moved {
        return;
    }

    let dest = player_transform.translation + (movement * 3.);

    let mut save_move = true;
    for (_, transform, info) in collider_query.iter() {
        let collision = collide(dest, Vec2::new(25., 25.), transform.translation, info.size);
        if collision.is_some() {
            save_move = false;
        }
    }

    if save_move {
        player_transform.translation = dest;
    }
}

pub fn setup_players(
    mut commands: Commands,
) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn().insert_bundle(UiCameraBundle::default());

    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(25.0, 25.0)),
            ..Sprite::default()
        },
        transform: Transform {
            translation: Vec3::new(0., 0., 10.),
            ..Transform::default()
        },
        ..SpriteBundle::default()
    };

    commands
        .spawn()
        .insert(Player {})
        .insert_bundle(sprite_bundle);
}
