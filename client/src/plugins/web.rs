use bevy::prelude::*;

pub struct WebPlugin {}

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        {
            app.add_system(update_window_size);
        }
    }
}


#[cfg(target_arch = "wasm32")]
fn update_window_size(mut windows: ResMut<Windows>) {
    //See: https://github.com/rust-windowing/winit/issues/1491
    // TODO: use window resize event instead of polling
    use approx::relative_eq;
    let web_window = web_sys::window().unwrap();
    let width = web_window.inner_width().unwrap().as_f64().unwrap() as f32;
    let height = web_window.inner_height().unwrap().as_f64().unwrap() as f32;

    let window = windows.get_primary_mut().unwrap();
    if relative_eq!(width, window.width()) && relative_eq!(height, window.height()) {
        return;
    }

    info!(
        "resizing canvas {:?}, old size {:?}",
        (width, height),
        (window.width(), window.height())
    );
    window.set_resolution(width, height);
}
