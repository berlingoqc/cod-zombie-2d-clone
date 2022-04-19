mod plugins;
mod config; mod map;
mod player;
mod game;

use bevy::asset::AssetServerSettings;
use bevy::{
    prelude::*,
    window::WindowDescriptor, sprite::collide_aabb::collide, core::FixedTimestep
};
use bevy_ecs_tilemap::prelude::*;
use player::{Player, Projectile, Velocity};

use crate::map::{MapPlugin, data::{MovementCollider, ProjectileCollider, MapElementPosition}};
use crate::{plugins::frame_cnt::FPSPlugin, game::{Game, GameState}};

const TIME_STEP: f32 = 1.0 / 60.0;



fn main() {
    let opts = config::Opts::get();
    info!("opts: {:?}", opts);


    let vsync = opts.fps == 60 && !opts.benchmark_mode;

    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Zombie".to_string(),
        width: 500.,
        height: 300.,
        resizable: true,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..WindowDescriptor::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(MapPlugin{})
    .add_startup_system(setup_camera);

    app
        .init_resource::<Game>()
        .add_state(GameState::Playing)
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(apply_velocity)
                .with_system(move_player)
                .with_system(movement_projectile)
                .with_system(fire)
        );
    

    //if opts.benchmark_mode {
    //    app.add_plugin(FPSPlugin{});
    //}

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    //commands.spawn_bundle(UiCameraBundle::default());
}

fn fire(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
}

fn apply_velocity(
    mut commands: Commands, mut query: Query<(&mut Transform, &Velocity, Entity)>
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

fn movement_projectile(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    collider_query: Query<(Entity, &Transform, &MapElementPosition), (With<ProjectileCollider>, With<MapElementPosition>, Without<Player>)>,
) {
    for (projectile_entity, transform) in projectile_query.iter() {
        for (_, transform_collider, info) in collider_query.iter() {
            let collision = collide(
                transform.translation,
                Vec2::new(5.,5.),
                transform_collider.translation,
                info.size
            );
            if let Some(collision) = collision {
                commands.entity(projectile_entity).despawn();
                break;
            }

        }
    }
}

fn move_player(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    collider_query: Query<(Entity, &Transform, &MapElementPosition), (With<MovementCollider>, With<MapElementPosition>, Without<Player>)>,
) {
    let mut player_transform = query.single_mut();

    let mut movement = Vec3::default();
    let mut moved = false;

    if keyboard_input.pressed(KeyCode::W) {
        movement += Vec3::new(0.,1.,0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::S) {
        movement += Vec3::new(0.,-1.,0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::A) {
        movement += Vec3::new(-1.,0.,0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::D) {
        movement += Vec3::new(1.,0.,0.);
        moved = true;
    }

    if keyboard_input.pressed(KeyCode::Space) {
        let amo_v = if !moved {
            Vec3::new(1., 0., 0.)
        } else {
            movement
        };
        commands
            .spawn()
            .insert(Projectile{})
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
            .insert(ProjectileCollider{})
            .insert(Velocity{ v: amo_v.truncate() * 500. });
    }



    if !moved { return; }

    let dest = player_transform.translation + (movement * 3.);

    let mut save_move = true;
    for (collider_entity, transform, info) in collider_query.iter() {
        let collision = collide(
            dest,
            Vec2::new(25.,25.),
            transform.translation,
            info.size
        );
        if let Some(collision) = collision {
            save_move = false;
        }
    }

    if save_move {
        player_transform.translation = dest;
    }
}

fn setup(
    mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>
) {
    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(25.0, 25.0)),
            ..Sprite::default()
        },
        transform: Transform {
           translation: Vec3::new(0.,0.,10.),
           ..Transform::default()
        },
        ..SpriteBundle::default()
    };

    commands
        .spawn()
        .insert(Player{})
        .insert_bundle(sprite_bundle);

}
