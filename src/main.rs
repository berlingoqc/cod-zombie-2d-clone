mod plugins;
mod config;

use bevy::{
    prelude::*,
    window::WindowDescriptor
};

use crate::plugins::frame_cnt::FPSPlugin;

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
        vsync,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..WindowDescriptor::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup);


    //if opts.benchmark_mode {
        app.add_plugin(FPSPlugin{});
    //}

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
 }
