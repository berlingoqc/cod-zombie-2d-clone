use bevy::{prelude::*, window::WindowDescriptor};

fn hello_world_system() {
    bevy::log::info!("Hello cruel world")
}

fn main() {
    let mut app = App::new();


    app.insert_resource(WindowDescriptor {
        title: "Zombie".to_string(),
        width: 500.,
        height: 300.,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..WindowDescriptor::default()
    })
    .add_plugins(DefaultPlugins)
    .add_system(hello_world_system)
    .run()
    
}
